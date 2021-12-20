mod select;
#[macro_use]
mod where_cond;

use select::SelectBuilder;

pub use where_cond::Where;

#[derive(Debug)]
pub struct QueryBuilder {}

impl QueryBuilder {
    pub fn select(table: &str) -> SelectBuilder {
        SelectBuilder::new(table)
    }

    pub fn insert() {}

    pub fn update() {}

    pub fn delete() {}
}
