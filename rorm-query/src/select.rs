use rorm_error::Result;

use crate::{Filter, Where};

#[derive(Debug, Default)]
pub struct SelectBuilder {
    table: String,
    columns: Vec<String>,
    filter: Filter,
}

impl SelectBuilder {
    pub fn new<S>(table: S) -> Self
    where
        S: ToString,
    {
        Self {
            table: table.to_string(),
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
    /// assert_eq!(&a, "SELECT a, b FROM ta ");
    /// ```
    pub fn column<S>(&mut self, col: S) -> &mut Self
    where
        S: ToString,
    {
        self.columns.push(col.to_string());
        self
    }

    /// Set columns
    ///
    /// # Examples
    ///
    /// ```
    /// use rorm_query::{QueryBuilder, sql_str};
    ///
    /// let a = QueryBuilder::select("ta")
    ///     .columns(&["a", "b"])
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(&a, "SELECT a, b FROM ta ");
    /// ```
    pub fn columns<T, S>(&mut self, cols: T) -> &mut Self
    where
        T: IntoIterator<Item = S>,
        S: ToString,
    {
        self.columns = cols
            .into_iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        self
    }

    /// Build sql
    pub fn build(&self) -> Result<String> {
        // Validate builder
        self.validate()?;

        let mut parts = Vec::<String>::new();

        // Build prefix
        parts.push("SELECT".into());

        // Build columns
        parts.push(self.columns.join(", "));

        // Build table
        parts.push("FROM".into());
        parts.push(self.table.clone());

        // Build filter
        parts.push(self.filter.build()?);

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

crate::lazy_impl_filer_for_struct! { SelectBuilder }
