use std::marker::PhantomData;

use crate::{
    error::Result, pool::Value, query::Where, Connection, DeleteBuilder, Entity, InsertBuilder,
    ToSqlParamPair, UpdateBuilder,
};

pub struct RepoTransaction<'conn, E: Entity> {
    pub conn: &'conn Connection,
    pairs: Vec<(String, Vec<Vec<Value>>)>,
    _marker: PhantomData<E>,
}

impl<'conn, E: Entity> RepoTransaction<'conn, E> {
    pub fn new(conn: &'conn Connection) -> Self {
        Self {
            conn,
            pairs: vec![],
            _marker: PhantomData,
        }
    }

    pub fn insert(&mut self) -> TransInsertBuilder<'conn, '_, E> {
        TransInsertBuilder::new(self)
    }

    pub fn delete(&mut self) -> TransDeleteBuilder<'conn, '_, E> {
        TransDeleteBuilder::new(self)
    }

    pub fn update(&mut self) -> TransUpdateBuilder<'conn, '_, E> {
        TransUpdateBuilder::new(self)
    }

    pub async fn commit(self) -> Result<()> {
        self.conn.execute_many(self.pairs).await?;
        Ok(())
    }

    pub async fn rollback(self) -> Result<()> {
        // Do nothing
        Ok(())
    }
}

pub struct TransInsertBuilder<'conn, 'tx, E: Entity> {
    tx: &'tx mut RepoTransaction<'conn, E>,
    builder: InsertBuilder<E>,
}

impl<'conn, 'tx, E: Entity> TransInsertBuilder<'conn, 'tx, E> {
    pub fn new(tx: &'tx mut RepoTransaction<'conn, E>) -> Self {
        Self {
            tx,
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

    pub async fn one(self) -> Result<()> {
        self.tx.pairs.extend(self.builder.to_sql_param_pair()?);
        Ok(())
    }

    pub async fn all(self) -> Result<()> {
        self.tx.pairs.extend(self.builder.to_sql_param_pair()?);
        Ok(())
    }
}

pub struct TransDeleteBuilder<'conn, 'tx, E: Entity> {
    tx: &'tx mut RepoTransaction<'conn, E>,
    builder: DeleteBuilder<E>,
}

impl<'conn, 'tx, E: Entity> TransDeleteBuilder<'conn, 'tx, E> {
    pub fn new(tx: &'tx mut RepoTransaction<'conn, E>) -> Self {
        Self {
            tx,
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
        self.tx
            .pairs
            .extend(self.builder.limit(limit, offset).to_sql_param_pair()?);
        Ok(())
    }

    pub async fn one(self) -> Result<()> {
        self.tx
            .pairs
            .extend(self.builder.limit(1, 0).to_sql_param_pair()?);
        Ok(())
    }

    pub async fn all(self) -> Result<()> {
        self.tx.pairs.extend(self.builder.to_sql_param_pair()?);
        Ok(())
    }
}

pub struct TransUpdateBuilder<'conn, 'tx, E: Entity> {
    tx: &'tx mut RepoTransaction<'conn, E>,
    builder: UpdateBuilder<E>,
}

impl<'conn, 'tx, E: Entity> TransUpdateBuilder<'conn, 'tx, E> {
    pub fn new(tx: &'tx mut RepoTransaction<'conn, E>) -> Self {
        Self {
            tx,
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
        self.tx
            .pairs
            .extend(self.builder.limit(limit, offset).to_sql_param_pair()?);
        Ok(())
    }

    pub async fn one(self) -> Result<()> {
        self.tx
            .pairs
            .extend(self.builder.limit(1, 0).to_sql_param_pair()?);
        Ok(())
    }

    pub async fn all(self) -> Result<()> {
        self.tx.pairs.extend(self.builder.to_sql_param_pair()?);
        Ok(())
    }
}
