mod connection;
mod drivers;
mod info;
mod value;

use std::collections::HashMap;

pub use connection::Connection;
pub use value::{FromValue, ToValue, Value};

pub mod driver {
    #[cfg(feature = "sqlite")]
    pub use rusqlite;
}

pub use info::{ColumnInfo, ColumnType, IndexInfo, IndexKeyInfo, TableInfo};

use rorm_error::Result;

#[async_trait::async_trait]
pub trait Driver: Sync + Send {
    async fn execute_many(&self, pairs: Vec<(String, Vec<Vec<Value>>)>) -> Result<Vec<u64>>; // Vec<(sql, params_list)>
    async fn query_many(&self, sql: &str, params: Vec<Value>) -> Result<Vec<Row>>;
    async fn init_table(&self, info: &TableInfo) -> Result<()>;
}

#[derive(Debug)]
pub struct Row {
    pub(crate) values: HashMap<String, Value>,
}

impl Row {
    pub fn get<T: FromValue<Output = T>>(&self, index: &str) -> Result<T> {
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
