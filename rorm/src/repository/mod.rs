mod builder;

use std::marker::PhantomData;

use crate::{error::Result, Connection, Entity};

use builder::{RepoDeleteBuilder, RepoFindBuilder, RepoInsertBuilder, RepoUpdateBuilder};

#[derive(Clone)]
pub struct Repository<E: Entity> {
    pub conn: Connection,
    _marker: PhantomData<E>,
}

impl<E: Entity> Repository<E> {
    #[inline]
    pub(crate) fn new(conn: Connection) -> Self {
        Self {
            conn,
            _marker: PhantomData,
        }
    }

    #[inline]
    pub async fn init(&self) -> Result<()> {
        E::init(&self.conn).await?;

        Ok(())
    }

    #[inline]
    pub fn insert(&self) -> RepoInsertBuilder<E> {
        RepoInsertBuilder::new(self.conn.clone())
    }

    #[inline]
    pub fn delete(&self) -> RepoDeleteBuilder<E> {
        RepoDeleteBuilder::new(self.conn.clone())
    }

    #[inline]
    pub fn update(&self) -> RepoUpdateBuilder<E> {
        RepoUpdateBuilder::new(self.conn.clone())
    }

    #[inline]
    pub fn find(&self) -> RepoFindBuilder<E> {
        RepoFindBuilder::new(self.conn.clone())
    }
}
