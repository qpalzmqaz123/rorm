use std::{future::Future, sync::Arc};

use crate::{Driver, Result, Row, TableInfo, Value};

#[derive(Clone)]
pub struct Connection {
    driver: Arc<dyn Driver>,
}

impl Connection {
    /// # Open connect
    ///
    /// Sqlite example:
    ///     - `connect("sqlite://memory")`
    ///     - `connect("sqlite:///tmp/db.sqlite")`
    pub async fn connect(url: &str) -> Result<Self> {
        #[cfg(feature = "sqlite")]
        if url.starts_with("sqlite://") {
            return Self::connect_sqlite(url);
        }

        Err(rorm_error::connection!("Unsupport url `{}`", url))
    }

    /// # Generate a dummy connection
    pub fn dummy() -> Self {
        Self {
            driver: Arc::new(DummyDriver),
        }
    }

    pub async fn execute_many(&self, pairs: Vec<(String, Vec<Vec<Value>>)>) -> Result<Vec<u64>> {
        Ok(self.driver.execute_many(pairs).await?)
    }

    pub async fn query_many_map<T, Fun, Fut>(
        &self,
        sql: &str,
        params: Vec<Value>,
        map: Fun,
    ) -> Result<Vec<T>>
    where
        Fun: Fn(Row) -> Fut,
        Fut: Future<Output = Result<T>>,
    {
        let rows = self.driver.query_many(sql, params).await?;
        let mut res_list = Vec::<T>::new();

        for row in rows {
            res_list.push(map(row).await?);
        }

        Ok(res_list)
    }

    pub async fn init_table(&self, info: &TableInfo) -> Result<()> {
        self.driver.init_table(info).await?;

        Ok(())
    }

    #[cfg(feature = "sqlite")]
    fn connect_sqlite(url: &str) -> Result<Self> {
        let path = &url[9..];
        let conn = if path == "memory" {
            rusqlite::Connection::open_in_memory()
                .map_err(|e| rorm_error::connection!("Sqlite open_in_memory error: {}", e))?
        } else {
            rusqlite::Connection::open(path)
                .map_err(|e| rorm_error::connection!("Sqlite open `{}` error: {}", path, e))?
        };
        let driver = crate::drivers::sqlite::SqliteConnProxy::new(conn);

        Ok(Self {
            driver: Arc::new(driver),
        })
    }
}

struct DummyDriver;

#[async_trait::async_trait]
impl Driver for DummyDriver {
    async fn execute_many(&self, _pairs: Vec<(String, Vec<Vec<Value>>)>) -> Result<Vec<u64>> {
        unreachable!()
    }

    async fn query_many(&self, _sql: &str, _params: Vec<Value>) -> Result<Vec<Row>> {
        unreachable!()
    }

    async fn init_table(&self, _info: &TableInfo) -> Result<()> {
        unreachable!()
    }
}
