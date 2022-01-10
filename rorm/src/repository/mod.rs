mod builder;
mod transaction;

use std::marker::PhantomData;

use crate::{error::Result, Connection, Entity};

use builder::{RepoDeleteBuilder, RepoFindBuilder, RepoInsertBuilder, RepoUpdateBuilder};
use transaction::RepoTransaction;

#[derive(Clone)]
pub struct Repository<E: Entity> {
    pub conn: Connection,
    _marker: PhantomData<E>,
}

impl<E: Entity + Send> Repository<E> {
    pub fn new(conn: Connection) -> Self {
        Self {
            conn,
            _marker: PhantomData,
        }
    }

    pub fn dummy() -> Self {
        Self {
            conn: Connection::dummy(),
            _marker: PhantomData,
        }
    }

    pub async fn init(&self) -> Result<()> {
        E::init(&self.conn).await?;

        Ok(())
    }

    pub fn insert(&self) -> RepoInsertBuilder<E> {
        RepoInsertBuilder::new(self.conn.clone())
    }

    pub fn delete(&self) -> RepoDeleteBuilder<E> {
        RepoDeleteBuilder::new(self.conn.clone())
    }

    pub fn update(&self) -> RepoUpdateBuilder<E> {
        RepoUpdateBuilder::new(self.conn.clone())
    }

    pub fn find(&self) -> RepoFindBuilder<E> {
        RepoFindBuilder::new(self.conn.clone())
    }

    pub fn transaction(&self) -> RepoTransaction<'_, E> {
        RepoTransaction::new(&self.conn)
    }
}
