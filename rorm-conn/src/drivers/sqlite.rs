//! # Sqlite driver
//!
//! Build:
//!   1. ./configure CC=x86_64-linux-musl-gcc --disable-shared --enable-static --disable-readline --disable-tcl
//!   2. OPTS=-DSQLITE_ENABLE_UPDATE_DELETE_LIMIT=1 make sqlite3.c

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use rorm_error::Result;

use crate::{ColumnType, Driver, IndexInfo, Row, TableInfo, Value};

#[cfg(feature = "runtime-tokio-0.2")]
use tokio_02::task::spawn_blocking;

#[derive(Clone)]
pub struct SqliteConnProxy {
    conn: Arc<Mutex<rusqlite::Connection>>,
}

impl SqliteConnProxy {
    pub fn new(conn: rusqlite::Connection) -> Self {
        Self {
            conn: Arc::new(Mutex::new(conn)),
        }
    }
}

#[async_trait::async_trait]
impl Driver for SqliteConnProxy {
    async fn execute_many(&self, pairs: Vec<(String, Vec<Vec<Value>>)>) -> Result<Vec<u64>> {
        let proxy = self.clone();
        let ids = spawn_blocking(move || {
            let mut conn = proxy
                .conn
                .lock()
                .map_err(|e| rorm_error::connection!("SqliteConnProxy lock error: {}", e))?;

            log::trace!("Start transaction");
            let tx = conn
                .transaction()
                .map_err(|e| rorm_error::database!("Start transaction error: {}", e))?;

            let mut ids = Vec::<u64>::new();
            for (sql, params_list) in pairs {
                log::trace!("Prepare execute many `{}`", sql);
                let mut stmt = tx
                    .prepare(&sql)
                    .map_err(|e| rorm_error::database!("Prepare error: {}, sql: `{}`", e, sql))?;

                for param in params_list {
                    log::trace!("Execute {:?}", param);

                    stmt.execute(&rorm_param_to_rusqlite_param(&param)[..])
                        .map_err(|e| rorm_error::database!("Execute error: {}", e))?;

                    // Insert id
                    ids.push(tx.last_insert_rowid() as u64);
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
        let proxy = self.clone();
        let rows = spawn_blocking(move || {
            let conn = proxy
                .conn
                .lock()
                .map_err(|e| rorm_error::connection!("SqliteConnProxy lock error: {}", e))?;

            log::trace!("Prepare query many `{}`", sql_string);
            let mut stmt = conn.prepare(&sql_string).map_err(|e| {
                rorm_error::database!("Prepare query many error: {}, sql: `{}`", e, sql_string)
            })?;

            log::trace!("Query many {:?}", params);
            let mut sql_rows = stmt
                .query(&rorm_param_to_rusqlite_param(&params)[..])
                .map_err(|e| rorm_error::database!("Query error: {}", e))?;
            let mut rows = Vec::<Row>::new();
            while let Ok(Some(row)) = sql_rows.next() {
                let row = rusqlite_row_to_rorm_row(row)?;
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
        let index_sqls = info
            .indexes
            .iter()
            .map(|idx| gen_create_index(info.name, idx))
            .collect::<Vec<_>>();

        // Execute sql
        let proxy = self.clone();
        spawn_blocking(move || {
            let conn = proxy
                .conn
                .lock()
                .map_err(|e| rorm_error::connection!("SqliteConnProxy lock error: {}", e))?;

            log::trace!("Execute `{}`", table_sql);
            conn.execute(&table_sql, []).map_err(|e| {
                rorm_error::database!("Create table error: {}, sql: `{}`", e, table_sql)
            })?;

            for index_sql in index_sqls {
                log::trace!("Execute `{}`", index_sql);
                conn.execute(&index_sql, []).map_err(|e| {
                    rorm_error::database!("Create table error: {}, sql: `{}`", e, index_sql)
                })?;
            }

            Result::Ok(())
        })
        .await
        .map_err(|e| rorm_error::runtime!("Tokio join error: {}", e))??;

        Ok(())
    }
}

fn rorm_param_to_rusqlite_param(params: &Vec<Value>) -> Vec<&'_ dyn rusqlite::ToSql> {
    params.iter().map(|v| v as &dyn rusqlite::ToSql).collect()
}

fn rusqlite_row_to_rorm_row<'s>(src: &rusqlite::Row<'s>) -> Result<Row> {
    use rusqlite::types::ValueRef;

    let stmt = src.as_ref();

    let mut values = HashMap::new();
    for i in 0..stmt.column_count() {
        let column_name = stmt
            .column_name(i)
            .map_err(|e| rorm_error::database!("Get column name error: {}", e))?
            .to_string();

        if let Ok(v) = src.get_ref(i) {
            let value = match v {
                ValueRef::Null => Value::Null,
                ValueRef::Integer(v) => Value::I64(v),
                ValueRef::Real(v) => Value::F64(v),
                ValueRef::Text(v) => {
                    Value::Str(String::from_utf8(v.to_vec()).unwrap_or(String::new()))
                }
                ValueRef::Blob(v) => Value::Bytes(v.to_vec()),
            };
            values.insert(column_name, value);
        } else {
            break;
        }
    }

    Ok(Row { values })
}

impl rusqlite::ToSql for Value {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        match &self {
            Value::Null => <Option<u8> as rusqlite::ToSql>::to_sql(&None),
            Value::Bool(v) => <bool as rusqlite::ToSql>::to_sql(v),
            Value::U8(v) => <u8 as rusqlite::ToSql>::to_sql(v),
            Value::I8(v) => <i8 as rusqlite::ToSql>::to_sql(v),
            Value::U16(v) => <u16 as rusqlite::ToSql>::to_sql(v),
            Value::I16(v) => <i16 as rusqlite::ToSql>::to_sql(v),
            Value::U32(v) => <u32 as rusqlite::ToSql>::to_sql(v),
            Value::I32(v) => <i32 as rusqlite::ToSql>::to_sql(v),
            Value::U64(v) => <u64 as rusqlite::ToSql>::to_sql(v),
            Value::I64(v) => <i64 as rusqlite::ToSql>::to_sql(v),
            Value::F32(v) => <f32 as rusqlite::ToSql>::to_sql(v),
            Value::F64(v) => <f64 as rusqlite::ToSql>::to_sql(v),
            Value::Str(v) => <String as rusqlite::ToSql>::to_sql(v),
            Value::Bytes(v) => <Vec<u8> as rusqlite::ToSql>::to_sql(v),
        }
    }
}

fn gen_create_table(info: &TableInfo) -> String {
    let cols = info
        .columns
        .iter()
        .map(|col| {
            format!(
                "{name} {ty} {prim_key} {auto_incr} {not_null} {default} {unique}",
                name = col.name,
                ty = column_type_to_sqlite_type(&col.ty),
                prim_key = if col.is_primary_key {
                    "PRIMARY KEY"
                } else {
                    ""
                },
                auto_incr = if col.is_auto_increment {
                    "AUTOINCREMENT"
                } else {
                    ""
                },
                not_null = if col.is_not_null { "NOT NULL" } else { "" },
                default = col
                    .default
                    .map(|def| format!("DEFAULT {}", def))
                    .unwrap_or("".into()),
                unique = if col.is_unique { "UNIQUE" } else { "" },
            )
        })
        .collect::<Vec<_>>();

    format!(
        "CREATE TABLE IF NOT EXISTS {table_name} ({cols})",
        table_name = info.name,
        cols = cols.join(", ")
    )
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

fn column_type_to_sqlite_type(col: &ColumnType) -> String {
    match col {
        ColumnType::Bool => "INTEGER".into(),
        ColumnType::I8 => "INTEGER".into(),
        ColumnType::U8 => "INTEGER".into(),
        ColumnType::I16 => "INTEGER".into(),
        ColumnType::U16 => "INTEGER".into(),
        ColumnType::I32 => "INTEGER".into(),
        ColumnType::U32 => "INTEGER".into(),
        ColumnType::I64 => "INTEGER".into(),
        ColumnType::U64 => "INTEGER".into(),
        ColumnType::F32 => "REAL".into(),
        ColumnType::F64 => "REAL".into(),
        ColumnType::Str(_) => "TEXT".into(),
        ColumnType::Bytes(_) => "BLOB".into(),
    }
}
