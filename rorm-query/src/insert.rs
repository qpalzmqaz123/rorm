use rorm_error::Result;

#[derive(Debug, Default)]
pub struct InsertBuilder {
    table: String,
    columns: Vec<String>,
}

impl InsertBuilder {
    pub fn new(table: &str) -> Self {
        Self {
            table: table.into(),
            ..Default::default()
        }
    }

    /// Set columns
    ///
    /// # Examples
    ///
    /// ```
    /// use rorm_query::QueryBuilder;
    ///
    /// let sql = QueryBuilder::insert("ta")
    ///     .columns(["a", "b", "c"])
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(&sql, "INSERT INTO ta (a, b, c)");
    /// ```
    pub fn columns<'a, T>(mut self, columns: T) -> Self
    where
        T: IntoIterator<Item = &'a str>,
    {
        self.columns = columns
            .into_iter()
            .map(|c| c.into())
            .collect::<Vec<String>>();
        self
    }

    /// Build sql
    pub fn build(self) -> Result<String> {
        // Validate builder
        self.validate()?;

        let mut parts = Vec::<String>::new();

        // Build prefix
        parts.push(format!("INSERT INTO {}", self.table));

        // Build columns
        parts.push(format!("({})", self.columns.join(", ")));

        Ok(parts.join(" "))
    }

    /// Validate builder
    fn validate(&self) -> Result<()> {
        if self.columns.len() == 0 {
            return Err(rorm_error::query_builder!("Insert empty columns"));
        }

        Ok(())
    }
}
