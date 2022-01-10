use std::marker::PhantomData;

use crate::{error::Result, query, Connection, Entity, Model, ToSqlParamPair, Value};

pub struct DeleteBuilder<E: Entity> {
    sql_builder: query::DeleteBuilder,
    params: Vec<Value>,
    _marker1: PhantomData<E>,
}

impl<E: Entity> DeleteBuilder<E> {
    pub fn new() -> Self {
        Self {
            sql_builder: query::QueryBuilder::delete(E::TABLE_NAME),
            params: vec![],
            _marker1: PhantomData,
        }
    }

    pub fn filter_model<I>(mut self, model: I) -> Self
    where
        I: Into<E::Model>,
    {
        let (cond, params) = model.into().gen_where_and_params();
        if let Some(cond) = cond {
            self.sql_builder.where_cond(cond);
            self.params = params;
        }
        self
    }

    pub fn filter(mut self, cond: query::Where, params: Vec<Value>) -> Self {
        self.sql_builder.where_cond(cond);
        self.params = params;
        self
    }

    pub fn group_by(mut self, col: &str) -> Self {
        self.sql_builder.group_by(col);
        self
    }

    pub fn order_by(mut self, col: &str, is_asc: bool) -> Self {
        self.sql_builder.order_by(col, is_asc);
        self
    }

    pub fn limit(mut self, limit: u64, offset: u64) -> Self {
        self.sql_builder.limit(limit, offset);
        self
    }

    pub async fn execute(self, conn: &Connection) -> Result<()> {
        let pairs = self.to_sql_param_pair()?;
        conn.execute_many(pairs).await?;

        Ok(())
    }
}

impl<E: Entity> ToSqlParamPair for DeleteBuilder<E> {
    fn to_sql_param_pair(self) -> Result<Vec<(String, Vec<Vec<Value>>)>> {
        let sql = self.sql_builder.build()?;
        let params = self.params;

        Ok(vec![(sql, vec![params])])
    }
}
