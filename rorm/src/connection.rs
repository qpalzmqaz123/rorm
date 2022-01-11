use std::future::Future;

use rorm_conn::Connection as InternalConn;

use crate::{error::Result, Row, TableInfo, Value};

#[derive(Clone)]
pub struct Connection {
    internal: InternalConn,
}

impl Connection {
    pub async fn connect(url: &str) -> Result<Self> {
        Ok(Self {
            internal: InternalConn::connect(url).await?,
        })
    }

    pub fn dummy() -> Self {
        Self {
            internal: InternalConn::dummy(),
        }
    }

    pub async fn execute_many(&self, pairs: Vec<(String, Vec<Vec<Value>>)>) -> Result<Vec<u64>> {
        self.internal.execute_many(pairs).await
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
        self.internal.query_many_map(sql, params, map).await
    }

    pub async fn init_table(&self, info: &TableInfo) -> Result<()> {
        self.internal.init_table(info).await
    }
}
