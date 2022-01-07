mod delete;
mod find;
mod insert;
mod update;

pub use delete::DeleteBuilder;
pub use find::FindBuilder;
pub use insert::InsertBuilder;
pub use update::UpdateBuilder;

use crate::{error::Result, pool::Value};

pub trait ToSqlParamPair {
    fn to_sql_param_pair(self) -> Result<Vec<(String, Vec<Vec<Value>>)>>;
}
