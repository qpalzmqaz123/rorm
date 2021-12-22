use rorm_error::Result;

use crate::Value;

#[derive(Debug, Default)]
pub struct InsertBuilder {
    table: String,
    columns: Vec<String>,
    values_list: Vec<Vec<String>>,
}

impl InsertBuilder {
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
    /// use rorm_query::{QueryBuilder, sql_str};
    ///
    /// let sql = QueryBuilder::insert("ta")
    ///     .column("a")
    ///     .column("b")
    ///     .column("c")
    ///     .values([1.into(), 2.into(), sql_str("abc")])
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(&sql, "INSERT INTO ta (a, b, c) VALUES (1, 2, 'abc')");
    /// ```
    pub fn column(&mut self, col: &str) -> &mut Self {
        self.columns.push(col.into());
        self
    }

    /// Set columns
    ///
    /// # Examples
    ///
    /// ```
    /// use rorm_query::{QueryBuilder, sql_str};
    ///
    /// let sql = QueryBuilder::insert("ta")
    ///     .columns(&["a", "b", "c"])
    ///     .values([1.into(), 2.into(), sql_str("abc")])
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(&sql, "INSERT INTO ta (a, b, c) VALUES (1, 2, 'abc')");
    /// ```
    pub fn columns(&mut self, cols: &[&str]) -> &mut Self {
        self.columns = cols.iter().map(|s| s.to_string()).collect::<Vec<String>>();
        self
    }

    /// Set values
    ///
    /// # Examples
    ///
    /// ```
    /// use rorm_query::{QueryBuilder, sql_str};
    ///
    /// let sql = QueryBuilder::insert("ta")
    ///     .column("a")
    ///     .column("b")
    ///     .column("c")
    ///     .values([1.into(), 2.into(), sql_str("abc")])
    ///     .values([10.into(), 20.into(), sql_str("asd")])
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(&sql, "INSERT INTO ta (a, b, c) VALUES (1, 2, 'abc'), (10, 20, 'asd')");
    /// ```
    pub fn values<T>(&mut self, values: T) -> &mut Self
    where
        T: IntoIterator<Item = Value>,
    {
        self.values_list.push(
            values
                .into_iter()
                .map(|v| v.to_string())
                .collect::<Vec<String>>(),
        );
        self
    }

    /// Build sql
    pub fn build(&self) -> Result<String> {
        // Validate builder
        self.validate()?;

        let mut parts = Vec::<String>::new();

        // Build prefix
        parts.push(format!("INSERT INTO {}", self.table));

        // Build columns
        parts.push(format!("({})", self.columns.join(", ")));

        // Build values
        parts.push("VALUES".into());
        parts.push(
            self.values_list
                .iter()
                .map(|values| format!("({})", values.join(", ")))
                .collect::<Vec<String>>()
                .join(", "),
        );

        Ok(parts.join(" "))
    }

    /// Validate builder
    fn validate(&self) -> Result<()> {
        if self.columns.len() == 0 {
            return Err(rorm_error::query_builder!("Insert empty columns"));
        }

        if self.values_list.len() == 0 {
            return Err(rorm_error::query_builder!("Empty values list"));
        }

        for values in &self.values_list {
            if values.len() != self.columns.len() {
                return Err(rorm_error::query_builder!(
                    "Columns and values length mismatch"
                ));
            }
        }

        Ok(())
    }
}
