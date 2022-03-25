use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use mysql_lib::prelude::Queryable;
use rorm_error::Result;

#[cfg(feature = "runtime-tokio-0.2")]
use tokio_02::task::spawn_blocking;

#[cfg(feature = "runtime-tokio-1")]
use tokio_1::task::spawn_blocking;

use crate::{ColumnInfo, ColumnType, Driver, IndexInfo, Row, TableInfo, Value};

pub struct MysqlConnProxy {
    conn: Arc<Mutex<mysql_lib::Conn>>,
}

impl MysqlConnProxy {
    pub fn new(conn: mysql_lib::Conn) -> Self {
        Self {
            conn: Arc::new(Mutex::new(conn)),
        }
    }
}

#[async_trait::async_trait]
impl Driver for MysqlConnProxy {
    async fn execute_many(&self, pairs: Vec<(String, Vec<Vec<Value>>)>) -> Result<Vec<u64>> {
        let conn = self.conn.clone();
        let ids = spawn_blocking(move || {
            let mut conn = conn
                .lock()
                .map_err(|e| rorm_error::connection!("MysqlConnProxy lock error: {}", e))?;

            log::trace!("Start transaction");
            let mut tx = conn
                .start_transaction(mysql_lib::TxOpts::default())
                .map_err(|e| rorm_error::database!("Start transaction error: {}", e))?;

            let mut ids = Vec::<u64>::new();
            for (sql, params_list) in pairs {
                log::trace!("Prepare execute many `{}`", sql);
                let stmt = tx
                    .prep(&sql)
                    .map_err(|e| rorm_error::database!("Prepare error: {}, sql: `{}`", e, sql))?;

                for param in params_list {
                    log::trace!("Execute {:?}", param);

                    tx.exec_drop(&stmt, param)
                        .map_err(|e| rorm_error::database!("Execute error: {}", e))?;

                    // Insert id
                    ids.push(tx.last_insert_id().unwrap_or_default() as u64);
                }
            }

            log::trace!("Commit transaction");
            tx.commit()
                .map_err(|e| rorm_error::database!("Commit error: {}", e))?;

            Result::Ok(ids)
        })
        .await
        .map_err(|e| rorm_error::runtime!("Tokio join error: {}", e))??;

        Ok(ids)
    }

    async fn query_many(&self, sql: &str, params: Vec<Value>) -> Result<Vec<Row>> {
        let sql_string = sql.to_string();
        let conn = self.conn.clone();
        let rows = spawn_blocking(move || {
            let mut conn = conn
                .lock()
                .map_err(|e| rorm_error::connection!("MysqlConnProxy lock error: {}", e))?;

            log::trace!("Prepare query many `{}`", sql_string);
            let stmt = conn.prep(&sql_string).map_err(|e| {
                rorm_error::database!("Prepare query many error: {}, sql: `{}`", e, sql_string)
            })?;

            log::trace!("Query many {:?}", params);
            let sql_rows = conn
                .exec_iter(&stmt, params)
                .map_err(|e| rorm_error::database!("Query error: {}", e))?;
            let mut rows = Vec::<Row>::new();
            for res in sql_rows {
                let mysql_row = res.map_err(|e| rorm_error::database!("Get row error: {}", e))?;
                let row = mysql_row_to_rorm_row(mysql_row)?;
                log::trace!("Append row: {:?}", row);
                rows.push(row);
            }

            Result::Ok(rows)
        })
        .await
        .map_err(|e| rorm_error::runtime!("Tokio join error: {}", e))??;

        Ok(rows)
    }

    async fn init_table(&self, info: &TableInfo) -> Result<()> {
        // Generate sql
        let table_sql = gen_create_table(info);
        let _index_sqls = info
            .indexes
            .iter()
            .map(|idx| gen_create_index(info.name, idx))
            .collect::<Vec<_>>();

        // Execute sql
        let conn = self.conn.clone();
        spawn_blocking(move || {
            let mut conn = conn
                .lock()
                .map_err(|e| rorm_error::connection!("MysqlConnProxy lock error: {}", e))?;

            log::trace!("Execute `{}`", table_sql);
            conn.query_drop(&table_sql).map_err(|e| {
                rorm_error::database!("Create table error: {}, sql: `{}`", e, table_sql)
            })?;

            // TODO: Add index
            // for index_sql in index_sqls {
            //     log::trace!("Execute `{}`", index_sql);
            //     conn.query_drop(&index_sql).map_err(|e| {
            //         rorm_error::database!("Create table error: {}, sql: `{}`", e, index_sql)
            //     })?;
            // }

            Result::Ok(())
        })
        .await
        .map_err(|e| rorm_error::runtime!("Tokio join error: {}", e))??;

        Ok(())
    }
}

fn mysql_row_to_rorm_row(src: mysql_lib::Row) -> Result<Row> {
    let mut values = HashMap::new();
    let cols = src.columns_ref();
    for i in 0..src.len() {
        let column_name = cols
            .get(i)
            .ok_or(rorm_error::database!(
                "Cannot get column name of index {}",
                i
            ))?
            .name_str()
            .to_string();

        if let Some(mysql_value) = src.as_ref(i) {
            let value = match mysql_value {
                mysql_lib::Value::NULL => Value::Null,
                mysql_lib::Value::Bytes(v) => Value::Bytes(v.clone()),
                mysql_lib::Value::Int(v) => Value::I64(*v),
                mysql_lib::Value::UInt(v) => Value::U64(*v),
                mysql_lib::Value::Float(v) => Value::F32(*v),
                mysql_lib::Value::Double(v) => Value::F64(*v),
                _ => {
                    return Err(rorm_error::database!(
                        "Unsupported mysql value type: {:?}",
                        mysql_value
                    ))
                }
            };

            values.insert(column_name, value);
        }
    }

    Ok(Row { values })
}

fn gen_create_table(info: &TableInfo) -> String {
    format!(
        "CREATE TABLE IF NOT EXISTS {table_name} ({cols})",
        table_name = info.name,
        cols = gen_cols(info.columns).join(", ")
    )
}

fn gen_cols(infos: &[ColumnInfo]) -> Vec<String> {
    let mut cols = vec![];

    for info in infos {
        if let Some(ref_info) = info.flatten_ref {
            cols.extend(gen_cols(ref_info.columns));
        } else {
            let col = format!(
                "{name} {ty} {prim_key} {auto_incr} {not_null} {default} {unique}",
                name = info.name,
                ty = column_type_to_mysql_type(&info.ty),
                prim_key = if info.is_primary_key {
                    "PRIMARY KEY"
                } else {
                    ""
                },
                auto_incr = if info.is_auto_increment {
                    "AUTO_INCREMENT"
                } else {
                    ""
                },
                not_null = if info.is_not_null { "NOT NULL" } else { "" },
                default = info
                    .default
                    .map(|def| format!("DEFAULT {}", def))
                    .unwrap_or("".into()),
                unique = if info.is_unique { "UNIQUE" } else { "" },
            );
            cols.push(col);
        }
    }

    cols
}

fn gen_create_index(table_name: &str, index_info: &IndexInfo) -> String {
    let cols = index_info
        .keys
        .iter()
        .map(|k| format!("{}", k.column_name))
        .collect::<Vec<_>>();

    format!(
        "CREATE INDEX IF NOT EXISTS {index_name} ON {table_name} ({cols})",
        index_name = index_info.name,
        table_name = table_name,
        cols = cols.join(", ")
    )
}

fn column_type_to_mysql_type(col: &ColumnType) -> String {
    match col {
        ColumnType::Bool => "TINYINT".into(),
        ColumnType::I8 => "TINYINT".into(),
        ColumnType::U8 => "TINYINT".into(),
        ColumnType::I16 => "SMALLINT".into(),
        ColumnType::U16 => "SMALLINT".into(),
        ColumnType::I32 => "INTEGER".into(),
        ColumnType::U32 => "INTEGER".into(),
        ColumnType::I64 => "BIGINT".into(),
        ColumnType::U64 => "BIGINT".into(),
        ColumnType::F32 => "FLOAT".into(),
        ColumnType::F64 => "DOUBLE".into(),
        ColumnType::Str(len) => {
            if *len <= 65535 {
                format!("VARCHAR({})", len)
            } else {
                "LONGTEXT".into()
            }
        }
        ColumnType::Bytes(len) => {
            if *len <= 65535 {
                "BLOB".into()
            } else {
                "LONGBLOB".into()
            }
        }
    }
}

impl From<Value> for mysql_lib::Value {
    fn from(v: Value) -> Self {
        match v {
            Value::Null => Self::NULL,
            Value::Bool(v) => Self::Int(v as _),
            Value::U8(v) => Self::UInt(v as _),
            Value::I8(v) => Self::Int(v as _),
            Value::U16(v) => Self::UInt(v as _),
            Value::I16(v) => Self::Int(v as _),
            Value::U32(v) => Self::UInt(v as _),
            Value::I32(v) => Self::Int(v as _),
            Value::U64(v) => Self::UInt(v as _),
            Value::I64(v) => Self::Int(v as _),
            Value::F32(v) => Self::Float(v),
            Value::F64(v) => Self::Double(v),
            Value::Str(v) => Self::Bytes(v.into_bytes()),
            Value::Bytes(v) => Self::Bytes(v),
        }
    }
}
