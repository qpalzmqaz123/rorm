use crate::{
    error::Result, pool::Value, query::Where, Connection, DeleteBuilder, Entity, FindBuilder,
    InsertBuilder, UpdateBuilder,
};

pub struct RepoInsertBuilder<E: Entity> {
    conn: Connection,
    builder: InsertBuilder<E>,
}

impl<E: Entity> RepoInsertBuilder<E> {
    pub fn new(conn: Connection) -> Self {
        Self {
            conn,
            builder: InsertBuilder::new(),
        }
    }

    pub fn model<I>(mut self, model: I) -> Self
    where
        I: Into<E::Model>,
    {
        self.builder = self.builder.model(model);
        self
    }

    pub fn models<L, I>(mut self, models: L) -> Self
    where
        L: IntoIterator<Item = I>,
        I: Into<E::Model>,
    {
        self.builder = self.builder.models(models);
        self
    }

    pub async fn one(self) -> Result<E::PrimaryKey> {
        Ok(self
            .builder
            .execute(&self.conn)
            .await?
            .into_iter()
            .next()
            .ok_or(crate::error::database!(
                "Repository insert one return empty ids"
            ))?)
    }

    pub async fn all(self) -> Result<Vec<E::PrimaryKey>> {
        Ok(self.builder.execute(&self.conn).await?)
    }
}

pub struct RepoDeleteBuilder<E: Entity> {
    conn: Connection,
    builder: DeleteBuilder<E>,
}

impl<E: Entity> RepoDeleteBuilder<E> {
    pub fn new(conn: Connection) -> Self {
        Self {
            conn,
            builder: DeleteBuilder::new(),
        }
    }

    pub fn filter_model<I>(mut self, model: I) -> Self
    where
        I: Into<E::Model>,
    {
        self.builder = self.builder.filter_model(model);
        self
    }

    pub fn filter(mut self, cond: Where, params: Vec<Value>) -> Self {
        self.builder = self.builder.filter(cond, params);
        self
    }

    pub fn group_by(mut self, col: &str) -> Self {
        self.builder = self.builder.group_by(col);
        self
    }

    pub fn order_by(mut self, col: &str, is_asc: bool) -> Self {
        self.builder = self.builder.order_by(col, is_asc);
        self
    }

    pub async fn limit(self, limit: u64, offset: u64) -> Result<()> {
        Ok(self
            .builder
            .limit(limit, offset)
            .execute(&self.conn)
            .await?)
    }

    pub async fn one(self) -> Result<()> {
        Ok(self.builder.limit(1, 0).execute(&self.conn).await?)
    }

    pub async fn all(self) -> Result<()> {
        Ok(self.builder.execute(&self.conn).await?)
    }
}

pub struct RepoUpdateBuilder<E: Entity> {
    conn: Connection,
    builder: UpdateBuilder<E>,
}

impl<E: Entity> RepoUpdateBuilder<E> {
    pub fn new(conn: Connection) -> Self {
        Self {
            conn,
            builder: UpdateBuilder::new(),
        }
    }

    pub fn set_model<I>(mut self, model: I) -> Self
    where
        I: Into<E::Model>,
    {
        self.builder = self.builder.set_model(model);
        self
    }

    pub fn filter_model<I>(mut self, model: I) -> Self
    where
        I: Into<E::Model>,
    {
        self.builder = self.builder.filter_model(model);
        self
    }

    pub fn filter(mut self, cond: Where, params: Vec<Value>) -> Self {
        self.builder = self.builder.filter(cond, params);
        self
    }

    pub fn group_by(mut self, col: &str) -> Self {
        self.builder = self.builder.group_by(col);
        self
    }

    pub fn order_by(mut self, col: &str, is_asc: bool) -> Self {
        self.builder = self.builder.order_by(col, is_asc);
        self
    }

    pub async fn limit(self, limit: u64, offset: u64) -> Result<()> {
        Ok(self
            .builder
            .limit(limit, offset)
            .execute(&self.conn)
            .await?)
    }

    pub async fn one(self) -> Result<()> {
        Ok(self.builder.limit(1, 0).execute(&self.conn).await?)
    }

    pub async fn all(self) -> Result<()> {
        Ok(self.builder.execute(&self.conn).await?)
    }
}

pub struct RepoFindBuilder<E: Entity> {
    conn: Connection,
    builder: FindBuilder<E>,
}

impl<E: Entity> RepoFindBuilder<E> {
    pub fn new(conn: Connection) -> Self {
        Self {
            conn,
            builder: FindBuilder::new(),
        }
    }

    pub fn filter_model<I>(mut self, model: I) -> Self
    where
        I: Into<E::Model>,
    {
        self.builder = self.builder.filter_model(model);
        self
    }

    pub fn filter(mut self, cond: Where, params: Vec<Value>) -> Self {
        self.builder = self.builder.filter(cond, params);
        self
    }

    pub fn group_by(mut self, col: &str) -> Self {
        self.builder = self.builder.group_by(col);
        self
    }

    pub fn order_by(mut self, col: &str, is_asc: bool) -> Self {
        self.builder = self.builder.order_by(col, is_asc);
        self
    }

    pub async fn limit(self, limit: u64, offset: u64) -> Result<Vec<E>> {
        Ok(self
            .builder
            .limit(limit, offset)
            .execute(&self.conn)
            .await?)
    }

    pub async fn one(self) -> Result<E> {
        let list = self.builder.limit(1, 0).execute(&self.conn).await?;

        list.into_iter()
            .next()
            .ok_or(crate::error::database!("Return empty rows"))
    }

    pub async fn all(self) -> Result<Vec<E>> {
        Ok(self.builder.execute(&self.conn).await?)
    }
}
