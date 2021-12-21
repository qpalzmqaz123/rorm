mod insert;
mod select;
#[macro_use]
mod where_cond;
mod value;

use insert::InsertBuilder;
use select::SelectBuilder;
use value::Value;

pub use value::sql_str;
pub use where_cond::Where;

#[derive(Debug)]
pub struct QueryBuilder {}

impl QueryBuilder {
    pub fn select(table: &str) -> SelectBuilder {
        SelectBuilder::new(table)
    }

    pub fn insert(table: &str) -> InsertBuilder {
        InsertBuilder::new(table)
    }

    pub fn update() {}

    pub fn delete() {}
}
