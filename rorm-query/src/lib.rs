mod delete;
mod filter;
mod insert;
mod query_value;
mod select;
mod update;
mod where_cond;

pub use delete::DeleteBuilder;
pub use insert::InsertBuilder;
pub use query_value::{sql_str, QueryValue};
pub use select::SelectBuilder;
pub use update::UpdateBuilder;
pub use where_cond::Where;

use filter::Filter;

#[derive(Debug)]
pub struct QueryBuilder {}

impl QueryBuilder {
    pub fn select(table: &str) -> SelectBuilder {
        SelectBuilder::new(table)
    }

    pub fn insert(table: &str) -> InsertBuilder {
        InsertBuilder::new(table)
    }

    pub fn update(table: &str) -> UpdateBuilder {
        UpdateBuilder::new(table)
    }

    pub fn delete(table: &str) -> DeleteBuilder {
        DeleteBuilder::new(table)
    }
}
