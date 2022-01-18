mod transaction;

use std::future::Future;

use rorm_conn::Connection as InternalConn;

use crate::{error::Result, Entity, Repository, Row, TableInfo, Value};

use transaction::Transaction;

#[derive(Clone)]
pub struct Connection {
    internal: InternalConn,
}

impl Connection {
    #[inline]
    pub async fn connect(url: &str) -> Result<Self> {
        Ok(Self {
            internal: InternalConn::connect(url).await?,
        })
    }

    #[inline]
    pub fn dummy() -> Self {
        Self {
            internal: InternalConn::dummy(),
        }
    }

    #[inline]
    pub async fn execute_one(&self, sql: &str, params: Vec<Value>) -> Result<u64> {
        self.internal.execute_one(sql, params).await
    }

    #[inline]
    pub async fn execute_many(&self, pairs: Vec<(String, Vec<Vec<Value>>)>) -> Result<Vec<u64>> {
        self.internal.execute_many(pairs).await
    }

    #[inline]
    pub async fn query_one_map<T, Fun, Fut>(
        &self,
        sql: &str,
        params: Vec<Value>,
        map: Fun,
    ) -> Result<T>
    where
        Fun: Fn(Row) -> Fut,
        Fut: Future<Output = Result<T>>,
    {
        self.internal.query_one_map(sql, params, map).await
    }

    #[inline]
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

    #[inline]
    pub async fn init_table(&self, info: &TableInfo) -> Result<()> {
        self.internal.init_table(info).await
    }

    #[inline]
    pub fn repository<E: Entity>(&self) -> Repository<E> {
        Repository::new(self.clone())
    }

    #[inline]
    pub fn transaction(&self) -> Transaction<'_> {
        Transaction::new(self)
    }
}
