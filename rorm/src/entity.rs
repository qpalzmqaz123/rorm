use crate::{
    error::Result, Connection, DeleteBuilder, FindBuilder, InsertBuilder, Model, Row, TableInfo,
    UpdateBuilder,
};

#[async_trait::async_trait]
pub trait Entity: Sized {
    type PrimaryKey;
    type Model: Model<Self::PrimaryKey>;

    /// Table name
    const TABLE_NAME: &'static str;

    /// Column names
    const COLUMNS: &'static [&'static str];

    /// Table info
    const INFO: TableInfo;

    /// Convert database row to self
    async fn from_row(conn: &Connection, row: Row) -> Result<Self>;

    /// Init table, create table and index if not exists
    async fn init(conn: &Connection) -> Result<()> {
        Ok(conn.init_table(&Self::INFO).await?)
    }

    /// Insert builder
    fn insert() -> InsertBuilder<Self> {
        InsertBuilder::new()
    }

    /// Delete builder
    fn delete() -> DeleteBuilder<Self> {
        DeleteBuilder::new()
    }

    /// Update builder
    fn update() -> UpdateBuilder<Self> {
        UpdateBuilder::new()
    }

    /// Find builder
    fn find() -> FindBuilder<Self> {
        FindBuilder::new()
    }
}
