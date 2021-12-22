use crate::{Driver, Result, Row, Value};

pub struct Connection {
    driver: Box<dyn Driver>,
}

impl Connection {
    pub fn new(driver: Box<dyn Driver>) -> Self {
        Self { driver }
    }

    pub async fn execute_one(&self, sql: &str, params: Vec<Value>) -> Result<u64> {
        Ok(self.driver.execute_one(sql, params).await?)
    }

    pub async fn execute_many(&self, sql: &str, params: Vec<Vec<Value>>) -> Result<Vec<u64>> {
        Ok(self.driver.execute_many(sql, params).await?)
    }

    pub async fn query_one_map<T, F>(&self, sql: &str, params: Vec<Value>, map: F) -> Result<T>
    where
        F: FnOnce(Row) -> Result<T>,
    {
        let row = self.driver.query_one(sql, params).await?;
        let res = map(row)?;

        Ok(res)
    }

    pub async fn query_many_map<T, F>(
        &self,
        sql: &str,
        params: Vec<Value>,
        map: F,
    ) -> Result<Vec<T>>
    where
        F: Fn(Row) -> Result<T>,
    {
        let rows = self.driver.query_many(sql, params).await?;
        let mut res_list = Vec::<T>::new();

        for row in rows {
            res_list.push(map(row)?);
        }

        Ok(res_list)
    }
}
