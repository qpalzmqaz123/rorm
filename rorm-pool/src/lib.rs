mod connection;
mod drivers;
mod value;

pub use connection::Connection;
pub use value::{FromValue, ToValue, Value};

#[cfg(feature = "sqlite")]
pub use drivers::sqlite;

use rorm_error::Result;

#[async_trait::async_trait]
pub trait Driver {
    async fn execute_one(&self, sql: &str, params: Vec<Value>) -> Result<u64>;
    async fn execute_many(&self, sql: &str, params: Vec<Vec<Value>>) -> Result<Vec<u64>>;
    async fn query_one(&self, sql: &str, params: Vec<Value>) -> Result<Row>;
    async fn query_many(&self, sql: &str, params: Vec<Value>) -> Result<Vec<Row>>;
}

#[derive(Debug)]
pub struct Row {
    pub(crate) values: Vec<Value>,
}

impl Row {
    pub fn get<T: FromValue>(&self, index: usize) -> Result<<T as FromValue>::Output> {
        if let Some(v) = self.values.get(index) {
            Ok(T::from_value(v)?)
        } else {
            Err(rorm_error::out_of_range!(
                "Index out of range: index: {}, values length: {}",
                index,
                self.values.len()
            ))
        }
    }
}
