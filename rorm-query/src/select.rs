use rorm_error::Result;

use crate::Where;

#[derive(Debug, Default)]
pub struct SelectBuilder {
    table: String,
    columns: Vec<String>,
    where_cond: Option<Where>,
}

impl SelectBuilder {
    pub fn new(table: &str) -> Self {
        Self {
            table: table.into(),
            ..Default::default()
        }
    }

    /// Append column
    ///
    /// # Examples
    ///
    /// ```
    /// use rorm_query::QueryBuilder;
    ///
    /// let a = QueryBuilder::select("ta")
    ///     .column("a")
    ///     .column("b")
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(&a, "SELECT a, b FROM ta");
    /// ```
    pub fn column(mut self, col: &str) -> Self {
        self.columns.push(col.into());
        self
    }

    /// Set where condition
    ///
    /// # Examples
    ///
    /// ```
    /// use rorm_query::{QueryBuilder, and, lt, gt};
    ///
    /// let sql = QueryBuilder::select("ta")
    ///     .column("a")
    ///     .where_cond(and!(gt!("a", 1), lt!("b", 5)))
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(&sql, "SELECT a FROM ta WHERE ((a > 1) AND (b < 5))");
    /// ```
    pub fn where_cond(mut self, cond: Where) -> Self {
        self.where_cond = Some(cond);
        self
    }

    /// Build sql
    pub fn build(self) -> Result<String> {
        // Validate builder
        self.validate()?;

        let mut parts = Vec::<String>::new();

        // Build prefix
        parts.push("SELECT".into());

        // Build columns
        parts.push(self.columns.join(", "));

        // Build table
        parts.push("FROM".into());
        parts.push(self.table);

        // Build where
        if let Some(whe) = &self.where_cond {
            parts.push("WHERE".into());
            parts.push(whe.to_string());
        }

        Ok(parts.join(" "))
    }

    /// Validate builder
    fn validate(&self) -> Result<()> {
        if self.columns.len() == 0 {
            return Err(rorm_error::query_builder!("Select empty columns"));
        }

        Ok(())
    }
}
