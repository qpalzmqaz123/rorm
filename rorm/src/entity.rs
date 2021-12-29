use crate::{error::Result, pool::Connection, pool::Row, FindOption};

#[async_trait::async_trait]
pub trait Entity: Sized {
    type PrimaryKey;
    type Model;

    /// Table name
    const TABLE_NAME: &'static str;

    /// Column names
    const COLUMNS: &'static [&'static str];

    /// Convert database row to self
    fn from_row(row: Row) -> Result<Self>;

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
    async fn update<SM, DM>(conn: &Connection, src: SM, dst: DM) -> Result<()>
    where
        SM: Into<Self::Model> + Send,
        DM: Into<Self::Model> + Send;

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
