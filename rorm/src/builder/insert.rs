use std::marker::PhantomData;

use rorm_query::QueryBuilder;

use crate::{
    error::Result,
    pool::{Connection, Value},
    Entity, Model,
};

use super::ToSqlParamPair;

pub struct InsertBuilder<E: Entity> {
    pairs: Vec<(Vec<&'static str>, Vec<Vec<Value>>)>, // (cols, params_list)
    _marker1: PhantomData<E>,
}

impl<E: Entity> InsertBuilder<E> {
    pub fn new() -> Self {
        Self {
            pairs: vec![],
            _marker1: PhantomData,
        }
    }

    pub fn model<I>(mut self, model: I) -> Self
    where
        I: Into<E::Model>,
    {
        let pairs = model.into().into_set_pairs();
        let (cols, params) =
            pairs
                .into_iter()
                .fold((vec![], vec![]), |(mut cols, mut params), (col, value)| {
                    cols.push(col);
                    params.push(value);
                    (cols, params)
                });

        // Check if columns same with last element of pairs
        if let Some((last_cols, last_pairs)) = self.pairs.last_mut() {
            if last_cols == &cols {
                // Append params to last paris
                last_pairs.push(params);

                return self;
            }
        }

        // Append new pair to pairs
        self.pairs.push((cols, vec![params]));

        self
    }

    pub fn models<L, I>(self, models: L) -> Self
    where
        L: IntoIterator<Item = I>,
        I: Into<E::Model>,
    {
        models
            .into_iter()
            .fold(self, |this, model| this.model(model))
    }

    pub async fn execute(self, conn: &Connection) -> Result<Vec<E::PrimaryKey>> {
        let pairs = self.to_sql_param_pair()?;
        let ids = conn.execute_many(pairs).await?;

        Ok(ids
            .into_iter()
            .map(|id| E::Model::to_primary_key(id))
            .collect())
    }
}

impl<E: Entity> ToSqlParamPair for InsertBuilder<E> {
    fn to_sql_param_pair(self) -> Result<Vec<(String, Vec<Vec<Value>>)>> {
        let mut list = vec![];

        for (cols, params_list) in self.pairs {
            let values = cols.iter().map(|_| "?".into()).collect::<Vec<_>>();
            let sql = QueryBuilder::insert(E::TABLE_NAME)
                .columns(cols)
                .values(values)
                .build()?;
            list.push((sql, params_list));
        }

        Ok(list)
    }
}
