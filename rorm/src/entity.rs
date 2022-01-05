use crate::{error::Result, pool::Connection, pool::Row, FindOption, TableInfo};

#[async_trait::async_trait]
pub trait Entity: Sized {
    type PrimaryKey;
    type Model;

    /// Table name
    const TABLE_NAME: &'static str;

    /// Column names
    const COLUMNS: &'static [&'static str];

    /// Table info
    const INFO: TableInfo;

    /// Convert database row to self
    async fn from_row(conn: &Connection, row: Row) -> Result<Self>;

    /// Init table, create table and index if not exists
    async fn init(conn: &Connection) -> Result<()>;

    /// Insert single value
    async fn insert<M>(conn: &Connection, model: M) -> Result<Self::PrimaryKey>
    where
        M: Into<Self::Model> + Send;

    /// Insert multiple values
    async fn insert_many<T, M>(conn: &Connection, models: T) -> Result<Vec<Self::PrimaryKey>>
    where
        T: IntoIterator<Item = M> + Send,
        M: Into<Self::Model> + Send;

    /// Delete single value
    async fn delete<M>(conn: &Connection, model: M) -> Result<()>
    where
        M: Into<Self::Model> + Send;

    /// Update single value
    async fn update<CM, SM>(conn: &Connection, condition: CM, set: SM) -> Result<()>
    where
        CM: Into<Self::Model> + Send,
        SM: Into<Self::Model> + Send;

    /// Find single value
    async fn find<M>(conn: &Connection, model: M, option: Option<FindOption>) -> Result<Self>
    where
        M: Into<Self::Model> + Send;

    /// Find multiple values
    async fn find_many<M>(
        conn: &Connection,
        model: M,
        option: Option<FindOption>,
    ) -> Result<Vec<Self>>
    where
        M: Into<Self::Model> + Send;
}
