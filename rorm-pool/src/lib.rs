mod drivers;
mod value;

#[cfg(feature = "sqlite")]
pub use drivers::sqlite;
pub use value::{FromValue, ToValue, Value};

use rorm_error::Result;

#[async_trait::async_trait]
pub trait Driver {
    async fn execute(&self, sql: &str, params: Vec<Value>) -> Result<()>;
    async fn execute_many(&self, sql: &str, params: Vec<Vec<Value>>) -> Result<()>;
    async fn query_map<F, T>(&self, sql: &str, params: Vec<Value>, map_fn: F) -> Result<Vec<T>>
    where
        T: Send + 'static,
        F: Fn(Row) -> Result<T> + Send + 'static;
}

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
