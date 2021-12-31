use std::marker::PhantomData;

use crate::{error::Result, pool::Connection, Entity, FindOption};

#[derive(Clone)]
pub struct Repository<E: Entity> {
    conn: Connection,
    _marker: PhantomData<E>,
}

impl<E: Entity> Repository<E> {
    pub fn new(conn: Connection) -> Self {
        Self {
            conn,
            _marker: PhantomData,
        }
    }

    pub async fn init(&self) -> Result<()> {
        E::init(&self.conn).await?;

        Ok(())
    }

    pub async fn insert<M>(&self, model: M) -> Result<E::PrimaryKey>
    where
        M: Into<E::Model> + Send,
    {
        Ok(E::insert(&self.conn, model).await?)
    }

    pub async fn insert_many<T, M>(&self, models: T) -> Result<Vec<E::PrimaryKey>>
    where
        T: IntoIterator<Item = M> + Send,
        M: Into<E::Model> + Send,
    {
        Ok(E::insert_many(&self.conn, models).await?)
    }

    pub async fn delete<M>(&self, model: M) -> Result<()>
    where
        M: Into<E::Model> + Send,
    {
        Ok(E::delete(&self.conn, model).await?)
    }

    pub async fn update<CM, SM>(&self, condition: CM, set: SM) -> Result<()>
    where
        CM: Into<E::Model> + Send,
        SM: Into<E::Model> + Send,
    {
        Ok(E::update(&self.conn, condition, set).await?)
    }

    pub async fn find<M>(&self, model: M, option: Option<FindOption>) -> Result<E>
    where
        M: Into<E::Model> + Send,
    {
        Ok(E::find(&self.conn, model, option).await?)
    }

    pub async fn find_many<M>(&self, model: M, option: Option<FindOption>) -> Result<Vec<E>>
    where
        M: Into<E::Model> + Send,
    {
        Ok(E::find_many(&self.conn, model, option).await?)
    }
}
