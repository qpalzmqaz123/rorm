use rorm_error::Result;

use crate::Where;

#[derive(Debug, Default)]
pub struct DeleteBuilder {
    table: String,
    where_cond: Option<Where>,
}

impl DeleteBuilder {
    pub fn new(table: &str) -> Self {
        Self {
            table: table.into(),
            ..Default::default()
        }
    }

    /// Set where condition
    ///
    /// # Examples
    ///
    /// ```
    /// use rorm_query::{QueryBuilder, and, lt, gt};
    ///
    /// let sql = QueryBuilder::delete("ta")
    ///     .where_cond(and!(gt!("a", 1), lt!("b", 5)))
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(&sql, "DELETE FROM ta WHERE ((a > 1) AND (b < 5))");
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
        parts.push("DELETE".into());

        // Build table
        parts.push("FROM".into());
        parts.push(self.table.clone());

        // Build where
        if let Some(whe) = &self.where_cond {
            parts.push("WHERE".into());
            parts.push(whe.to_string());
        }

        Ok(parts.join(" "))
    }

    /// Validate builder
    fn validate(&self) -> Result<()> {
        Ok(())
    }
}
