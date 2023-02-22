//! Fcss 驱动
//! Fcss 仅支持 kv 形式，所以这里需解析 sql，适配 kv 形式
//! 转换后的形式为：$tableName_$index: {
//!     $col1Name: $jsonstr,
//!     $col2Name: $jsonstr,
//! }
//! 使用表名 + index 作为 key，value 按照列作为 key 存储 object，建表时手动设置的 index 中选取，因为 index 可以设置多个，
//! 所以选首个数组数量为 1 的列作为 index，例如 index 列表为 [[col1, col2], [col3], [col4]]，则 index 为 col3
//! FIXME: 在涉及参数的情况下，参数和 sql 中 ? 的顺序都是固定的，暂时按照顺序访问参数，以后再优化
//!     insert 形如：INSERT INTO $table (col1, col2) VALUES (?, ?)，参数顺序为: [col1, col2]
//!     delete 形如：DELETE FROM $table WHERE col1 = ? and col2 = ?，参数顺序为: [col1, col2]
//!     update 形如：UPDATE $table SET col1 = ?, col2 = ? WHERE col3 = ? and col4 = ?，参数顺序为：[col1, col2, col3, col4]
//!     find 形如：SELECT col1, col2 FROM $table WHERE col3 = ? and col4 = ?，参数顺序为：[col3, col4]

mod grpc;

use std::{borrow::Cow, collections::HashMap, slice::Iter, sync::RwLock};

use rorm_error::Result;
use serde_json::{json, Map as JsonMap, Value as JsonValue};
use sqlparser::{
    ast::{
        Assignment, BinaryOperator, Expr, Ident, ObjectName, Offset, OffsetRows, OrderByExpr,
        Query, Select, SetExpr, Statement, TableFactor, TableWithJoins, Value as SqlParserValue,
    },
    dialect::GenericDialect,
    parser::Parser,
};

use crate::{Driver, Row, TableInfo, Value};

use self::grpc::FcssGrpcClient;

/// 判断为真时报错不支持
macro_rules! check_not_support {
    ($cond:expr, $item:expr, $sql:expr) => {
        if $cond {
            return Err(rorm_error::argument!(
                "Fcss not support '{}', sql: {}",
                $item,
                $sql
            ));
        }
    };
}

pub struct FcssConn {
    client: FcssGrpcClient,
    table_infos: RwLock<Vec<FcssTableInfo>>, // Fcss 使用场景表和列不会很多，数组比 hash 快
}

impl FcssConn {
    pub async fn connect(uri_str: &str) -> Result<Self> {
        Ok(Self {
            client: FcssGrpcClient::connect(uri_str).await?,
            table_infos: Default::default(),
        })
    }

    /// 锁 table_infos，调用回调，方便调用的时候处理锁的报错
    fn table_infos_read_op<R, F>(&self, op: F) -> Result<R>
    where
        F: FnOnce(&Vec<FcssTableInfo>) -> Result<R>,
    {
        let lock = self
            .table_infos
            .read()
            .map_err(|e| rorm_error::database!("Fcss table infos read lock error: {}", e))?;
        op(&lock)
    }

    /// 获取对应 table 的索引列名称，该列会放到 fcss 的 key 中，用于 get 查询
    fn get_index_col(&self, table_name: &str) -> Result<&'static str> {
        self.table_infos_read_op(|infos| {
            infos
                .iter()
                .find_map(|info| (info.name == table_name).then_some(info.index))
                .ok_or(rorm_error::database!(
                    "Fcss cannot get index column of table '{}'",
                    table_name
                ))
        })
    }

    /// 执行单条 sql，用于增删改
    async fn execute_one(&self, sql: &str, params_list: Vec<Vec<Value>>) -> Result<Vec<u64>> {
        let stmt = parse_single_sql(sql)?;

        // 根据 sql 类型，分别执行增删改
        log::trace!(
            "Fcss execute sql: '{}', params count: {}",
            sql,
            params_list.len()
        );
        match stmt {
            Statement::Insert {
                or,
                into,
                table_name,
                columns,
                overwrite,
                source,
                partitioned,
                after_columns,
                table,
                on,
                returning,
            } => {
                check_not_support!(or.is_some(), "or syntax", sql);
                check_not_support!(!into, "!into syntax", sql);
                check_not_support!(overwrite, "overwrite syntax", sql);
                check_not_support!(partitioned.is_some(), "partitioned syntax", sql);
                check_not_support!(!after_columns.is_empty(), "after_columns syntax", sql);
                check_not_support!(table, "table syntax", sql);
                check_not_support!(on.is_some(), "on syntax", sql);
                check_not_support!(returning.is_some(), "returning syntax", sql);

                self.insert_with_params_list(&table_name, &columns, &source, &params_list)
                    .await?;
            }
            Statement::Delete {
                table_name,
                using,
                selection,
                returning,
            } => {
                check_not_support!(using.is_some(), "using syntax", sql);
                check_not_support!(returning.is_some(), "returning syntax", sql);

                self.delete_with_params_list(table_name, &selection, &params_list, sql)
                    .await?;
            }
            Statement::Update {
                table,
                assignments,
                from,
                selection,
                returning,
            } => {
                check_not_support!(from.is_some(), "from syntax", sql);
                check_not_support!(returning.is_some(), "returning syntax", sql);

                self.update_with_params_list(table, assignments, selection, &params_list, sql)
                    .await?;
            }
            _ => {
                return Err(rorm_error::argument!(
                    "Fcss execute does not support sql: '{}'",
                    sql
                ))
            }
        }

        Ok(params_list.iter().map(|_| 0).collect::<Vec<_>>())
    }

    /// 新增数据，传入参数列表，可以插入多行
    async fn insert_with_params_list(
        &self,
        table_name: &ObjectName,
        columns: &Vec<Ident>,
        source: &Query,
        params_list: &Vec<Vec<Value>>,
    ) -> Result<()> {
        let table_name = table_name.to_string();
        let index_col = self.get_index_col(&table_name)?;
        for params in params_list {
            self.insert_with_params(&table_name, index_col, columns, source, params)
                .await?;
        }

        Ok(())
    }

    /// 删除数据，匹配到 where 的都会被删除，在 fcss 场景下，先根据 where 查出所有匹配的数据，然后再逐个删除
    async fn delete_with_params_list(
        &self,
        table_name: TableFactor,
        cond: &Option<Expr>,
        params_list: &Vec<Vec<Value>>,
        sql: &str,
    ) -> Result<()> {
        let table_name = require_table_name_from_table_factor(&table_name, sql)?;
        let index_col = self.get_index_col(&table_name)?;
        for params in params_list {
            // 先根据条件筛选数据
            let delete_json_list = self.query_filter(&table_name, cond, params).await?;
            for delete_json in delete_json_list {
                // 删除数据
                let id_json = delete_json.get(index_col).ok_or(rorm_error::runtime!(
                    "Fcss cannot get id from queried data, index col: `{}`, json: `{:?}`",
                    index_col,
                    delete_json
                ))?;
                let id_str = gen_fcss_key(&table_name, &require_index_from_json_value(id_json)?);
                self.client.del(id_str).await?;
            }
        }

        Ok(())
    }

    /// 新增输入，传入参数，只插入一行
    async fn insert_with_params(
        &self,
        table_name: &str,
        index_col: &str,
        columns: &Vec<Ident>,
        source: &Query,
        params: &Vec<Value>,
    ) -> Result<()> {
        log::trace!("Fcss insert with params: {:?}", params);

        let mut values_iter = match source.body.as_ref() {
            SetExpr::Values(values) => (values.rows.len() == 1)
                .then_some(values.rows.first())
                .flatten()
                .ok_or_else(|| rorm_error::argument!("Fcss only supports insert 1 values row"))?
                .iter(),
            _ => return Err(rorm_error::argument!("Fcss insert expect values")),
        };
        let mut params_iter = params.iter();

        let mut json_map = JsonMap::new();
        let mut index_value = None;
        for column in columns {
            // 获取对应列的 json 值
            let column_name = &column.value;
            let value = values_iter.next().ok_or_else(|| {
                rorm_error::argument!(
                    "Fcss cannot get value of column '{}' in values",
                    column_name
                )
            })?;
            match value {
                Expr::Value(sql_parser_value) => {
                    let json_value =
                        sql_parser_and_param_value_to_json(sql_parser_value, &mut params_iter)?;

                    // 如果对应列为 index，同时提取 value 用于后面 key 生成
                    if column_name == index_col {
                        index_value = Some(require_index_from_json_value(&json_value)?);
                    }

                    json_map.insert(column_name.clone(), json_value);
                }
                _ => {
                    return Err(rorm_error::argument!(
                        "Fcss only supports value type in values"
                    ))
                }
            }
        }

        // 保存到设备
        let key = gen_fcss_key(
            table_name,
            &index_value.ok_or_else(|| {
                rorm_error::argument!("Fcss insert must be contains index column: '{}'", index_col)
            })?,
        );
        let json_value = JsonValue::Object(json_map);
        log::trace!(
            "Fcss grpc insert, key: '{}', value: '{}'",
            key,
            serde_json::to_string_pretty(&json_value).unwrap_or_default()
        );
        self.client
            .set(key, serde_json::to_string(&json_value).unwrap_or_default())
            .await?;

        Ok(())
    }

    /// 传入参数列表，修改数据，按照参数列表数量执行多少次，每次可以影响多行
    async fn update_with_params_list(
        &self,
        table: TableWithJoins,
        assignments: Vec<Assignment>,
        selection: Option<Expr>,
        params_list: &Vec<Vec<Value>>,
        sql: &str,
    ) -> Result<()> {
        let table_name = require_table_name_from_table_factor(&table.relation, sql)?;
        let index_col = self.get_index_col(&table_name)?;
        for params in params_list {
            self.update_with_params(&table_name, index_col, &assignments, &selection, params)
                .await?;
        }

        Ok(())
    }

    /// 传入参数列表，修改数据，按照参数执行，每次可以影响多行
    async fn update_with_params(
        &self,
        table_name: &str,
        index_col: &str,
        assignments: &[Assignment],
        selection: &Option<Expr>,
        params: &Vec<Value>,
    ) -> Result<()> {
        // 先处理赋值语句，获取需设置的列和值
        let (set_map, remain_params) = assignments_to_set_map(assignments, params)?;
        log::trace!(
            "Fcss update set map: {:?}, remain params: {:?}",
            set_map,
            remain_params
        );

        // 根据 where 条件，查询所有影响的行
        let rows = self
            .query_filter(table_name, selection, &remain_params)
            .await?;

        // 每行都修改
        for mut row in rows {
            // 获取 index 值
            let index_value = row
                .get(index_col)
                .ok_or(rorm_error::runtime!("Cannot get index from row: {:?}", row))?;

            // 生成 fcss key
            let key = gen_fcss_key(table_name, &require_index_from_json_value(index_value)?);

            // 修改 row
            for (col, col_value) in &set_map {
                row.insert(col.clone(), col_value.clone());
            }

            // 将 row 转成 json 字符串
            let value = serde_json::to_string(&JsonValue::Object(row)).map_err(|e| {
                rorm_error::runtime!("Fcss cannot convert json value to string: {}", e)
            })?;

            // 写入 fcss
            self.client.set(key, value).await?;
        }

        Ok(())
    }

    /// 查询数据，传入 where，limit 等条件，先根据 where 查询出所有匹配数据，再进行 offset，limit 筛选
    async fn query_filter_limit(
        &self,
        table_name: &str,
        cond: &Option<Expr>,
        order: &Vec<OrderByExpr>,
        offset: &Option<Offset>,
        limit: &Option<Expr>,
        sql: &str,
        params: Vec<Value>,
    ) -> Result<Vec<JsonMap<String, JsonValue>>> {
        check_not_support!(!order.is_empty(), "order by syntax", sql);

        let offset = get_offset(offset, sql)?;
        let limit = get_limit(limit, sql)?;

        // 通过 where 条件筛选所有数据
        let json_maps = self.query_filter(table_name, cond, &params).await?;

        // 通过 offset 和 limit 筛选
        let filtered = json_maps
            .into_iter()
            .skip(offset)
            .take(limit)
            .collect::<Vec<_>>();

        Ok(filtered)
    }

    /// 查询数据，根据 where 条件返回所有匹配的数据
    async fn query_filter(
        &self,
        table_name: &str,
        cond: &Option<Expr>,
        params: &Vec<Value>,
    ) -> Result<Vec<JsonMap<String, JsonValue>>> {
        let index_col = self.get_index_col(table_name)?;

        // 根据是否有 index，决定查一个还是查所有
        let all_fcss_entries = if cond
            .as_ref()
            .map(|ex| first_expr_is_index_eq(&ex, index_col))
            .unwrap_or(false)
        {
            // 传入索引，只查一个
            let index_param = params.first().ok_or(rorm_error::argument!(
                "Fcss cannot get index value from first parameter"
            ))?;
            let index_str = require_index_param(index_param)?;
            log::trace!(
                "Fcss lookup one from table `{}`, index: `{}`",
                table_name,
                index_str
            );
            let key = gen_fcss_key(table_name, &index_str);
            let value = self.client.get(&key).await?;

            vec![(key, value)]
        } else {
            // 未传入索引或条件，查所有
            log::trace!("Fcss lookup all from table `{}`", table_name);
            self.client.list().await?
        };

        // 过滤结果，fcss 目前只支持 getall，所以需要把其他表的隔离出来
        let mut fcss_entries_with_table = all_fcss_entries
            .into_iter()
            .filter(|(k, _)| key_is_match_table(k, table_name))
            .collect::<Vec<_>>();

        // 给结果排序，方便用户使用
        fcss_entries_with_table.sort_by(|a, b| a.0.cmp(&b.0));

        let json_maps = fcss_entries_with_table
            .into_iter()
            .map(|(_, value_str)| parse_json_object(&value_str))
            .collect::<Result<_>>()?;

        // 根据条件，筛选数据
        let filtered = if let Some(cond) = cond {
            let mut maps = vec![];
            for map in json_maps {
                if json_to_bool(filter_by_cond(cond, &map, &mut params.iter())?.as_ref()) {
                    maps.push(map);
                }
            }

            maps
        } else {
            json_maps
        };

        Ok(filtered)
    }
}

#[async_trait::async_trait]
impl Driver for FcssConn {
    async fn execute_many(&self, pairs: Vec<(String, Vec<Vec<Value>>)>) -> Result<Vec<u64>> {
        let mut all_ids = vec![];
        for (sql, params_list) in pairs {
            let ids = self.execute_one(&sql, params_list).await?;
            all_ids.extend(ids);
        }

        Ok(all_ids)
    }

    async fn query_many(&self, sql: &str, params: Vec<Value>) -> Result<Vec<Row>> {
        let stmt = parse_single_sql(sql)?;

        // 根据 sql 类型，执行查
        log::trace!("Fcss query sql: '{}', param count: {}", sql, params.len());
        match stmt {
            Statement::Query(query) => {
                check_not_support!(query.with.is_some(), "with syntax", sql);
                check_not_support!(query.fetch.is_some(), "fetch syntax", sql);
                check_not_support!(!query.locks.is_empty(), "locks syntax", sql);

                let limit = &query.limit;
                let offset = &query.offset;
                let order_by = &query.order_by;
                let (r#where, table_name) = match query.body.as_ref() {
                    SetExpr::Select(sel) => {
                        check_not_support!(sel.distinct, "distinct syntax", sql);
                        check_not_support!(sel.top.is_some(), "top syntax", sql);

                        (&sel.selection, get_table_name_from_select(sel, sql)?)
                    }
                    _ => {
                        return Err(rorm_error::argument!(
                            "Fcss only supports select body in query, sql: '{}'",
                            sql
                        ))
                    }
                };

                let json_maps = self
                    .query_filter_limit(&table_name, r#where, order_by, offset, limit, sql, params)
                    .await?;
                Ok(json_maps
                    .into_iter()
                    .map(|v| json_map_to_row(v))
                    .collect::<Result<_>>()?)
            }
            _ => Err(rorm_error::argument!(
                "Fcss query does not support sql: '{}'",
                sql
            )),
        }
    }

    /// 初始化表的时候，取出表的索引，存下来，用于后续操作的时候获取 key
    async fn init_table(&self, info: &TableInfo) -> Result<()> {
        log::trace!("Fcss init table: {:?}", info);

        let index = info
            .indexes
            .iter()
            .find_map(|v| {
                (v.keys.len() == 1)
                    .then_some(v.keys.get(0).map(|info| info.column_name))
                    .flatten()
            })
            .ok_or_else(|| rorm_error::argument!("Fcss table needs a single index"))?;
        log::trace!("Fcss table '{}' key column is '{}'", info.name, index);

        if let Ok(mut v) = self.table_infos.write() {
            v.push(FcssTableInfo {
                name: info.name,
                index,
            });
        }

        Ok(())
    }
}

struct FcssTableInfo {
    pub name: &'static str,
    pub index: &'static str,
}

/// 根据 sql 和传入的 param 转换为 json，如果 sql 为基础值（数字，字符串等），则直接转为 json，
/// 如果 sql 为 ?，则去迭代 param，将其转为 json
fn sql_parser_and_param_value_to_json(
    sql_parser_value: &SqlParserValue,
    params_iter: &mut Iter<Value>,
) -> Result<JsonValue> {
    let value = match sql_parser_value {
        SqlParserValue::Number(n_str, _) => {
            if let Ok(n) = n_str.parse::<i64>() {
                json!(n)
            } else if let Ok(n) = n_str.parse::<f64>() {
                json!(n)
            } else {
                return Err(rorm_error::argument!("Fcss cannot parse number: {}", n_str));
            }
        }
        SqlParserValue::SingleQuotedString(s) => {
            json!(s)
        }
        SqlParserValue::DoubleQuotedString(s) => {
            json!(s)
        }
        SqlParserValue::Boolean(b) => {
            json!(b)
        }
        SqlParserValue::Placeholder(p) => {
            if p != "?" {
                return Err(rorm_error::argument!("Fcss invalid placeholder: {}", p));
            }
            let param = params_iter
                .next()
                .ok_or(rorm_error::argument!("Fcss cannot get param"))?;
            param_value_to_json(param)?
        }
        _ => {
            return Err(rorm_error::argument!(
                "Fcss not support sql value: {:?}",
                sql_parser_value
            ))
        }
    };

    Ok(value)
}

/// 将 param 转成 json
fn param_value_to_json(param: &Value) -> Result<JsonValue> {
    let v = match param {
        Value::Null => JsonValue::Null,
        Value::Bool(b) => json!(b),
        Value::U8(n) => json!(n),
        Value::I8(n) => json!(n),
        Value::U16(n) => json!(n),
        Value::I16(n) => json!(n),
        Value::U32(n) => json!(n),
        Value::I32(n) => json!(n),
        Value::U64(n) => json!(n),
        Value::I64(n) => json!(n),
        Value::F32(n) => json!(n),
        Value::F64(n) => json!(n),
        Value::Str(s) => json!(s),
        _ => {
            return Err(rorm_error::argument!(
                "Fcss not support bytes param: {:?}",
                param
            ))
        }
    };

    Ok(v)
}

/// 传入 json 值，取出 index 的值，目前 fcss 仅支持 index 为字符串
fn require_index_from_json_value(value: &JsonValue) -> Result<String> {
    match value {
        JsonValue::String(s) => Ok(s.clone()),
        _ => Err(rorm_error::argument!(
            "Fcss index must be string, received json: '{:?}'",
            value
        )),
    }
}

/// 传入 param，取出 index 的值，目前 fcss 仅支持 index 为字符串
fn require_index_param(value: &Value) -> Result<String> {
    match value {
        Value::Str(s) => Ok(s.clone()),
        _ => Err(rorm_error::argument!(
            "Fcss index must be string, received param: '{:?}'",
            value
        )),
    }
}

/// 解析一条 sql，如果传入多条则报错
fn parse_single_sql(sql: &str) -> Result<Statement> {
    // 解析 sql，获取第一条 sql 语法树，fcss 不支持一次执行多条 sql
    let dialect = GenericDialect {};
    let stmts = Parser::parse_sql(&dialect, sql)
        .map_err(|e| rorm_error::argument!("Fcss parse sql '{}' error: {}", sql, e))?;
    if stmts.len() > 1 {
        return Err(rorm_error::argument!(
            "Fcss does not supports multiple sql executions at once: '{}'",
            sql
        ));
    }
    let stmt = stmts
        .into_iter()
        .next()
        .ok_or_else(|| rorm_error::argument!("Fcss received empty sql: '{}'", sql))?;

    Ok(stmt)
}

/// 从 select 语句中获取 table 名
fn get_table_name_from_select(query: &Select, sql: &str) -> Result<String> {
    check_not_support!(query.from.len() > 1, "multiple from fields", sql);
    let from = query.from.first().ok_or_else(|| {
        rorm_error::argument!("Fcss query cannot get table from query sql: '{}'", sql)
    })?;
    check_not_support!(!from.joins.is_empty(), "join syntax", sql);

    require_table_name_from_table_factor(&from.relation, sql)
}

/// 将 json object 转换为 rorm row
fn json_map_to_row(map: JsonMap<String, JsonValue>) -> Result<Row> {
    let mut res = HashMap::new();
    for (k, v) in map {
        res.insert(k, json_value_to_rorm_value(v)?);
    }

    Ok(Row { values: res })
}

/// 将 json value 转换为 rorm value
fn json_value_to_rorm_value(json_value: JsonValue) -> Result<Value> {
    let v = match json_value {
        JsonValue::Null => Value::Null,
        JsonValue::Bool(b) => Value::Bool(b),
        JsonValue::Number(ref n) => {
            if let Some(n) = n.as_u64() {
                Value::U64(n)
            } else if let Some(n) = n.as_i64() {
                Value::I64(n)
            } else if let Some(n) = n.as_f64() {
                Value::F64(n)
            } else {
                return Err(rorm_error::from_value!(
                    "Fcss cannot parse numbered json value to rorm value: '{:?}'",
                    json_value
                ));
            }
        }
        JsonValue::String(s) => Value::Str(s),
        _ => {
            return Err(rorm_error::from_value!(
                "Fcss cannot parse json value: {:?}",
                json_value
            ))
        }
    };

    Ok(v)
}

/// 获取 sql 中 offset 的值，目前仅支持数字，不支持参数等其他复杂情况
fn get_offset(offset: &Option<Offset>, sql: &str) -> Result<usize> {
    if let Some(offset) = offset {
        check_not_support!(!matches!(offset.rows, OffsetRows::None), "offset rows", sql);
        match &offset.value {
            Expr::Value(v) => match v {
                SqlParserValue::Number(n, _) => n.parse().map_err(|e| {
                    rorm_error::argument!(
                        "Fcss offset parse sql number error: '{}', sql: '{}'",
                        e,
                        sql
                    )
                }),
                _ => {
                    return Err(rorm_error::argument!(
                        "Fcss offset only supports number value in sql: '{}'",
                        sql
                    ))
                }
            },
            _ => {
                return Err(rorm_error::argument!(
                    "Fcss offset only supports value expr in sql: '{}'",
                    sql
                ))
            }
        }
    } else {
        Ok(0)
    }
}

/// 获取 sql 中 limit 的值，目前仅支持数字，不支持参数等其他复杂情况
fn get_limit(limit: &Option<Expr>, sql: &str) -> Result<usize> {
    if let Some(limit) = limit {
        match limit {
            Expr::Value(v) => match v {
                SqlParserValue::Number(n, _) => n.parse().map_err(|e| {
                    rorm_error::argument!(
                        "Fcss limit parse sql number error: '{}', sql: '{}'",
                        e,
                        sql
                    )
                }),
                _ => {
                    return Err(rorm_error::argument!(
                        "Fcss limit only supports number value in sql: '{}'",
                        sql
                    ))
                }
            },
            _ => {
                return Err(rorm_error::argument!(
                    "Fcss limit only supports value expr in sql: '{}'",
                    sql
                ))
            }
        }
    } else {
        Ok(usize::MAX)
    }
}

/// 判断第一个语句是否是 index 的表达式，用于通过 index 查询时的场景
fn first_expr_is_index_eq(expr: &Expr, index: &str) -> bool {
    match expr {
        Expr::Nested(n) => first_expr_is_index_eq(n, index),
        Expr::BinaryOp { left, op, right } => {
            *op == BinaryOperator::Eq && (left.to_string() == index || right.to_string() == index)
        }
        _ => false,
    }
}

/// 传入 json 对象和 param，根据表达式，递归执行，返回 json，用于 where 条件判断
fn filter_by_cond<'a>(
    expr: &Expr,
    json_map: &'a JsonMap<String, JsonValue>,
    param_iter: &mut Iter<Value>,
) -> Result<Cow<'a, JsonValue>> {
    let value = match expr {
        Expr::Identifier(ident) => json_map
            .get(&ident.value)
            .map(|v| Cow::Borrowed(v))
            .ok_or_else(|| rorm_error::argument!("Fcss invalid column: '{}'", ident.value))?,
        Expr::BinaryOp { left, op, right } => {
            let l_json = filter_by_cond(left, json_map, param_iter)?;
            let r_json = filter_by_cond(right, json_map, param_iter)?;
            Cow::Owned(binop(&l_json, &r_json, op)?)
        }
        Expr::Nested(ex) => filter_by_cond(ex, json_map, param_iter)?,
        Expr::Value(v) => {
            sql_parser_and_param_value_to_json(v, param_iter).map(|v| Cow::Owned(v))?
        }
        _ => {
            return Err(rorm_error::argument!(
                "Fcss not support filter syntax: '{}'",
                expr.to_string()
            ))
        }
    };

    Ok(value)
}

/// 执行 binop，目前仅支持等于判断
fn binop(l: &JsonValue, r: &JsonValue, op: &BinaryOperator) -> Result<JsonValue> {
    let value = match op {
        BinaryOperator::Eq => json!(l == r),
        _ => {
            return Err(rorm_error::database!(
                "Fcss binop not support operator: '{:?}'",
                op
            ))
        }
    };

    Ok(value)
}

/// 将 json 值转换为 bool 类型
fn json_to_bool(value: &JsonValue) -> bool {
    match value {
        JsonValue::Null => false,
        JsonValue::Bool(b) => *b,
        JsonValue::Number(n) => {
            if let Some(n) = n.as_u64() {
                n != 0
            } else if let Some(n) = n.as_i64() {
                n != 0
            } else if let Some(n) = n.as_f64() {
                n != 0.0
            } else {
                false
            }
        }
        JsonValue::String(s) => !s.is_empty(),
        JsonValue::Array(v) => !v.is_empty(),
        JsonValue::Object(o) => !o.is_empty(),
    }
}

/// FIXME: Fcss 计算 hash 用
fn times33(data: &str) -> u32 {
    data.bytes()
        .fold(5381, |hash, b| hash.wrapping_mul(33) + u32::from(b))
}

/// 生成 fcss 的 key
fn gen_fcss_key(table_name: &str, index: &str) -> String {
    // FIXME: 演示版本 fcss 限制，只能用 mac key，如 1.1.1
    // 所以这里 key 的结构为 {table hash 2B} + {index hash 4B}，hash 均使用 times33 算法，计算 32 位数字，table hash 取低 16 位
    let table_hash = times33(table_name) as u16;
    let index_hash = times33(index);

    let key = format!(
        "{:04x}.{:04x}.{:04x}",
        table_hash,
        (index_hash >> 16) as u16,
        index_hash as u16
    );

    log::trace!("Fcss generate key `{}.{}` -> `{}`", table_name, index, key);

    key
}

/// 传入 table 名，判断 key 是否属于表
fn key_is_match_table(key: &str, table_name: &str) -> bool {
    // FIXME: fcss 限制，目前使用 hash 来匹配
    let table_hash = times33(table_name) as u16;
    let key_prefix = format!("{:x}.", table_hash);
    key.starts_with(&key_prefix)
}

/// 将字符串转换为 json 对象，在查询 fcss 数据后使用
fn parse_json_object(s: &str) -> Result<JsonMap<String, JsonValue>> {
    let val: JsonValue = serde_json::from_str(s)
        .map_err(|e| rorm_error::runtime!("Fcss parse json object error: {}, raw: `{}`", e, s))?;
    match val {
        JsonValue::Object(map) => Ok(map),
        _ => Err(rorm_error::runtime!(
            "Fcss rpc get result value must be object, received: `{}`",
            val
        )),
    }
}

/// 解析 table factor，目前仅支持传入特定的 table 名，不支持其他复杂情况，如函数等
fn require_table_name_from_table_factor(factor: &TableFactor, sql: &str) -> Result<String> {
    match factor {
        TableFactor::Table {
            name,
            alias,
            args,
            with_hints,
        } => {
            check_not_support!(alias.is_some(), "alias syntax", sql);
            check_not_support!(args.is_some(), "function syntax", sql);
            check_not_support!(!with_hints.is_empty(), "with hints syntax", sql);

            Ok(name.to_string())
        }
        _ => Err(rorm_error::argument!(
            "Fcss only support table table factor, sql: '{}'",
            sql
        )),
    }
}

/// 根据 update 语句中 set 数据，生成以列为 key 的 map，同时返回 params
fn assignments_to_set_map(
    assignments: &[Assignment],
    params: &[Value],
) -> Result<(HashMap<String, JsonValue>, Vec<Value>)> {
    let mut set_map = HashMap::new();
    let mut params_iter = params.iter();
    for assignments in assignments {
        // 获取列名
        let col_name = assignments.id[0].value.clone();

        // 获取对应列名的值，有可能来自 sql 中写死，也可能为 ?，从 params 中获取
        let json_value = match &assignments.value {
            Expr::Value(value) => sql_parser_and_param_value_to_json(&value, &mut params_iter)?,
            _ => {
                return Err(rorm_error::argument!(
                    "Fcss update only supports value type in assignments expr"
                ))
            }
        };

        // 将列名与值存入 map
        set_map.insert(col_name, json_value);
    }

    // 前面可能消费了部分 params，这里将剩余的 params 收集起来，返回回去
    let remain_params = params_iter.cloned().collect::<Vec<_>>();

    Ok((set_map, remain_params))
}
