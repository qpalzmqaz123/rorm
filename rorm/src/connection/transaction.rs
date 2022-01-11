use std::marker::PhantomData;

use crate::{
    error::Result, query::Where, Connection, DeleteBuilder, Entity, InsertBuilder, ToSqlParamPair,
    UpdateBuilder, Value,
};

pub struct Transaction<'conn> {
    conn: &'conn Connection,
    pairs: Vec<(String, Vec<Vec<Value>>)>,
}

impl<'conn> Transaction<'conn> {
    pub fn new(conn: &'conn Connection) -> Self {
        Self {
            conn,
            pairs: vec![],
        }
    }

    pub fn repository<E: Entity>(&mut self) -> RepoTransaction<'_, E> {
        RepoTransaction::new(&mut self.pairs)
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

pub struct RepoTransaction<'pair, E: Entity> {
    pairs: &'pair mut Vec<(String, Vec<Vec<Value>>)>,
    _marker: PhantomData<E>,
}

impl<'conn, 'pair, E: Entity> RepoTransaction<'pair, E> {
    pub fn new(pairs: &'pair mut Vec<(String, Vec<Vec<Value>>)>) -> Self {
        Self {
            pairs,
            _marker: PhantomData,
        }
    }

    pub fn insert(&mut self) -> TransInsertBuilder<'_, E> {
        TransInsertBuilder::new(self.pairs)
    }

    pub fn delete(&mut self) -> TransDeleteBuilder<'_, E> {
        TransDeleteBuilder::new(self.pairs)
    }

    pub fn update(&mut self) -> TransUpdateBuilder<'_, E> {
        TransUpdateBuilder::new(self.pairs)
    }
}

pub struct TransInsertBuilder<'pair, E: Entity> {
    pairs: &'pair mut Vec<(String, Vec<Vec<Value>>)>,
    builder: InsertBuilder<E>,
}

impl<'conn, 'pair, E: Entity> TransInsertBuilder<'pair, E> {
    pub fn new(pairs: &'pair mut Vec<(String, Vec<Vec<Value>>)>) -> Self {
        Self {
            pairs,
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
        self.pairs.extend(self.builder.to_sql_param_pair()?);
        Ok(())
    }

    pub async fn all(self) -> Result<()> {
        self.pairs.extend(self.builder.to_sql_param_pair()?);
        Ok(())
    }
}

pub struct TransDeleteBuilder<'pair, E: Entity> {
    pairs: &'pair mut Vec<(String, Vec<Vec<Value>>)>,
    builder: DeleteBuilder<E>,
}

impl<'conn, 'pair, E: Entity> TransDeleteBuilder<'pair, E> {
    pub fn new(pairs: &'pair mut Vec<(String, Vec<Vec<Value>>)>) -> Self {
        Self {
            pairs,
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
        self.pairs
            .extend(self.builder.limit(limit, offset).to_sql_param_pair()?);
        Ok(())
    }

    pub async fn one(self) -> Result<()> {
        self.pairs
            .extend(self.builder.limit(1, 0).to_sql_param_pair()?);
        Ok(())
    }

    pub async fn all(self) -> Result<()> {
        self.pairs.extend(self.builder.to_sql_param_pair()?);
        Ok(())
    }
}

pub struct TransUpdateBuilder<'pair, E: Entity> {
    pairs: &'pair mut Vec<(String, Vec<Vec<Value>>)>,
    builder: UpdateBuilder<E>,
}

impl<'conn, 'pair, E: Entity> TransUpdateBuilder<'pair, E> {
    pub fn new(pairs: &'pair mut Vec<(String, Vec<Vec<Value>>)>) -> Self {
        Self {
            pairs,
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
        self.pairs
            .extend(self.builder.limit(limit, offset).to_sql_param_pair()?);
        Ok(())
    }

    pub async fn one(self) -> Result<()> {
        self.pairs
            .extend(self.builder.limit(1, 0).to_sql_param_pair()?);
        Ok(())
    }

    pub async fn all(self) -> Result<()> {
        self.pairs.extend(self.builder.to_sql_param_pair()?);
        Ok(())
    }
}
