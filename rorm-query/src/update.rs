use rorm_error::Result;

use crate::{Value, Where};

#[derive(Debug, Default)]
pub struct UpdateBuilder {
    table: String,
    kvs: Vec<(String, String)>,
    where_cond: Option<Where>,
}

impl UpdateBuilder {
    pub fn new<S>(table: S) -> Self
    where
        S: ToString,
    {
        Self {
            table: table.to_string(),
            ..Default::default()
        }
    }

    /// Append kv pair
    ///
    /// # Examples
    ///
    /// ```
    /// use rorm_query::{QueryBuilder, sql_str};
    ///
    /// let sql = QueryBuilder::update("ta")
    ///     .set("a", 1.into())
    ///     .set("b", sql_str("abc"))
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(&sql, "UPDATE ta SET a = 1, b = 'abc'");
    /// ```
    pub fn set<S>(&mut self, col: S, val: Value) -> &mut Self
    where
        S: ToString,
    {
        self.kvs.push((col.to_string(), val.to_string()));
        self
    }

    /// Append kv pair list
    ///
    /// # Examples
    ///
    /// ```
    /// use rorm_query::{QueryBuilder, sql_str};
    ///
    /// let sql = QueryBuilder::update("ta")
    ///     .sets([("a", 1.into()), ("b", sql_str("abc"))])
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(&sql, "UPDATE ta SET a = 1, b = 'abc'");
    /// ```
    pub fn sets<T, S>(&mut self, kvs: T) -> &mut Self
    where
        T: IntoIterator<Item = (S, Value)>,
        S: ToString,
    {
        self.kvs = kvs
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        self
    }

    /// Set where condition
    ///
    /// # Examples
    ///
    /// ```
    /// use rorm_query::{QueryBuilder, sql_str, and, lt, gt};
    ///
    /// let sql = QueryBuilder::update("ta")
    ///     .sets([("a", 1.into()), ("b", sql_str("abc"))])
    ///     .where_cond(and!(gt!("a", 1), lt!("b", 5)))
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(&sql, "UPDATE ta SET a = 1, b = 'abc' WHERE ((a > 1) AND (b < 5))");
    /// ```
    pub fn where_cond(&mut self, cond: Where) -> &mut Self {
        self.where_cond = Some(cond);
        self
    }

    /// Build sql
    pub fn build(&self) -> Result<String> {
        // Validate builder
        self.validate()?;

        let mut parts = Vec::<String>::new();

        // Build prefix
        parts.push("UPDATE".into());
        parts.push(self.table.clone());

        // Build kvs
        parts.push("SET".into());
        parts.push(
            self.kvs
                .iter()
                .map(|(k, v)| format!("{} = {}", k, v))
                .collect::<Vec<_>>()
                .join(", "),
        );

        // Build where
        if let Some(whe) = &self.where_cond {
            parts.push("WHERE".into());
            parts.push(whe.to_string());
        }

        Ok(parts.join(" "))
    }

    /// Validate builder
    fn validate(&self) -> Result<()> {
        if self.kvs.len() == 0 {
            return Err(rorm_error::query_builder!("Update empty columns"));
        }

        Ok(())
    }
}
