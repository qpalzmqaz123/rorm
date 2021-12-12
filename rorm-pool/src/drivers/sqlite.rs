use std::path::Path;

use r2d2_sqlite::{rusqlite, SqliteConnectionManager};
use rorm_error::Result;

use crate::{Driver, Row, Value};

#[cfg(feature = "runtime-tokio-0.2")]
use tokio_02::task::spawn_blocking;

pub struct Builder {
    mgr: SqliteConnectionManager,
    r2d2_builder: r2d2::Builder<SqliteConnectionManager>,
}

impl Builder {
    pub fn memory() -> Self {
        Self {
            mgr: SqliteConnectionManager::memory(),
            r2d2_builder: r2d2::Builder::new(),
        }
    }

    pub fn file<P: AsRef<Path>>(path: P) -> Self {
        Self {
            mgr: SqliteConnectionManager::file(path),
            r2d2_builder: r2d2::Builder::new(),
        }
    }

    pub fn max_size(mut self, max_size: u32) -> Self {
        self.r2d2_builder = self.r2d2_builder.max_size(max_size);
        self
    }

    pub fn build(self) -> Result<impl Driver> {
        let pool = self
            .r2d2_builder
            .build(self.mgr)
            .map_err(|e| rorm_error::connection!("SQLite connection error: {}", e))?;

        Ok(SqlitePoolProxy::new(pool))
    }
}

pub struct SqlitePoolProxy {
    pool: r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>,
}

impl SqlitePoolProxy {
    pub fn new(pool: r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl Driver for SqlitePoolProxy {
    async fn execute(&self, sql: &str, params: Vec<Value>) -> Result<()> {
        let sql_string = sql.to_string();
        let pool = self.pool.clone();
        let _res = spawn_blocking(move || {
            log::trace!("Get connection from pool");
            let conn = pool
                .get()
                .map_err(|e| rorm_error::timeout!("Get connection from pool timeout: {}", e))?;

            log::trace!("Execute single `{}`, {:?}", sql_string, params);
            let sqlite_param: Vec<&dyn rusqlite::ToSql> =
                params.iter().map(|v| v as &dyn rusqlite::ToSql).collect();
            conn.execute(&sql_string, &sqlite_param[..])
                .map_err(|e| rorm_error::database!("Execute error: {}", e))?;

            Result::<()>::Ok(())
        })
        .await
        .map_err(|e| rorm_error::runtime!("Tokio join error: {}", e))??;

        Ok(())
    }

    async fn execute_many(&self, sql: &str, params: Vec<Vec<Value>>) -> Result<()> {
        let sql_string = sql.to_string();
        let pool = self.pool.clone();
        let _res = spawn_blocking(move || {
            log::trace!("Get connection from pool");
            let mut conn = pool
                .get()
                .map_err(|e| rorm_error::timeout!("Get connection from pool timeout: {}", e))?;

            log::trace!("Start transaction");
            let tx = conn
                .transaction()
                .map_err(|e| rorm_error::database!("Start transaction error: {}", e))?;

            log::trace!("Prepare many `{}`", sql_string);
            let mut stmt = tx.prepare(&sql_string).map_err(|e| {
                rorm_error::database!("Prepare error: {}, sql: `{}`", e, sql_string)
            })?;

            for param in params {
                log::trace!("Execute {:?}", param);

                let sqlite_param: Vec<&dyn rusqlite::ToSql> =
                    param.iter().map(|v| v as &dyn rusqlite::ToSql).collect();
                stmt.execute(&sqlite_param[..])
                    .map_err(|e| rorm_error::database!("Execute error: {}", e))?;
            }

            log::trace!("Commit transaction");
            drop(stmt);
            tx.commit()
                .map_err(|e| rorm_error::database!("Commit error: {}", e))?;

            Result::<()>::Ok(())
        })
        .await
        .map_err(|e| rorm_error::runtime!("Tokio join error: {}", e))??;

        Ok(())
    }

    async fn query_map<F, T>(&self, sql: &str, params: Vec<Value>, map_fn: F) -> Result<Vec<T>>
    where
        T: Send + 'static,
        F: Fn(Row) -> Result<T> + Send + 'static,
    {
        let sql_string = sql.to_string();
        let pool = self.pool.clone();
        let list = spawn_blocking(move || {
            log::trace!("Get connection from pool");
            let conn = pool
                .get()
                .map_err(|e| rorm_error::timeout!("Get connection from pool timeout: {}", e))?;

            log::trace!("Prepare query `{}`", sql_string);
            let mut stmt = conn.prepare(&sql_string).map_err(|e| {
                rorm_error::database!("Prepare query error: {}, sql: `{}`", e, sql_string)
            })?;

            log::trace!("Query {:?}", params);
            let sqlite_param: Vec<&dyn rusqlite::ToSql> =
                params.iter().map(|v| v as &dyn rusqlite::ToSql).collect();
            let mut rows = stmt
                .query(&sqlite_param[..])
                .map_err(|e| rorm_error::database!("Query error: {}", e))?;
            let mut res = Vec::<T>::new();
            while let Ok(Some(row)) = rows.next() {
                let row = rusqlite_row_to_rorm_row(row);
                res.push(map_fn(row)?);
            }

            Result::<Vec<T>>::Ok(res)
        })
        .await
        .map_err(|e| rorm_error::runtime!("Tokio join error: {}", e))??;

        Ok(list)
    }
}

fn rusqlite_row_to_rorm_row<'s>(src: &rusqlite::Row<'s>) -> Row {
    use rusqlite::types::ValueRef;

    let mut values = Vec::<Value>::new();
    for i in 0..usize::MAX {
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
            values.push(value);
        } else {
            break;
        }
    }

    Row { values }
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
