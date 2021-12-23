use rorm_error::Result;

use crate::Where;

#[derive(Debug, Default)]
pub struct SelectBuilder {
    table: String,
    columns: Vec<String>,
    where_cond: Option<Where>,
    group_bys: Vec<String>,
    order_bys: Vec<(String, bool)>, // (column, is_asc)
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
    /// let a = QueryBuilder::select("ta")
    ///     .columns(&["a", "b"])
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(&a, "SELECT a, b FROM ta");
    /// ```
    pub fn columns(&mut self, cols: &[&str]) -> &mut Self {
        self.columns = cols.iter().map(|s| s.to_string()).collect::<Vec<String>>();
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
    pub fn where_cond(&mut self, cond: Where) -> &mut Self {
        self.where_cond = Some(cond);
        self
    }

    /// Append group by
    ///
    /// # Examples
    ///
    /// ```
    /// use rorm_query::{QueryBuilder, and, lt, gt};
    ///
    /// let sql = QueryBuilder::select("ta")
    ///     .column("a")
    ///     .where_cond(and!(gt!("a", 1), lt!("b", 5)))
    ///     .group_by("a")
    ///     .group_by("b")
    ///     .order_by("a", true)
    ///     .order_by("b", false)
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(&sql, "SELECT a FROM ta WHERE ((a > 1) AND (b < 5)) GROUP BY a, b ORDER BY a ASC, b DESC");
    /// ```
    pub fn group_by(&mut self, col: &str) -> &mut Self {
        self.group_bys.push(col.into());
        self
    }

    /// Set group by list
    ///
    /// # Examples
    ///
    /// ```
    /// use rorm_query::{QueryBuilder, and, lt, gt};
    ///
    /// let sql = QueryBuilder::select("ta")
    ///     .column("a")
    ///     .where_cond(and!(gt!("a", 1), lt!("b", 5)))
    ///     .group_bys(["a", "b"])
    ///     .order_by("a", true)
    ///     .order_by("b", false)
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(&sql, "SELECT a FROM ta WHERE ((a > 1) AND (b < 5)) GROUP BY a, b ORDER BY a ASC, b DESC");
    /// ```
    pub fn group_bys<'a, T>(&mut self, list: T) -> &mut Self
    where
        T: IntoIterator<Item = &'a str>,
    {
        self.group_bys = list.into_iter().map(|v| v.into()).collect();
        self
    }

    /// Append order by
    ///
    /// # Examples
    ///
    /// ```
    /// use rorm_query::{QueryBuilder, and, lt, gt};
    ///
    /// let sql = QueryBuilder::select("ta")
    ///     .column("a")
    ///     .where_cond(and!(gt!("a", 1), lt!("b", 5)))
    ///     .order_by("a", true)
    ///     .order_by("b", false)
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(&sql, "SELECT a FROM ta WHERE ((a > 1) AND (b < 5)) ORDER BY a ASC, b DESC");
    /// ```
    pub fn order_by(&mut self, col: &str, is_asc: bool) -> &mut Self {
        self.order_bys.push((col.into(), is_asc));
        self
    }

    /// Set order by list
    ///
    /// # Examples
    ///
    /// ```
    /// use rorm_query::{QueryBuilder, and, lt, gt};
    ///
    /// let sql = QueryBuilder::select("ta")
    ///     .column("a")
    ///     .where_cond(and!(gt!("a", 1), lt!("b", 5)))
    ///     .order_bys([("a", true), ("b", false)])
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(&sql, "SELECT a FROM ta WHERE ((a > 1) AND (b < 5)) ORDER BY a ASC, b DESC");
    /// ```
    pub fn order_bys<'a, T>(&mut self, list: T) -> &mut Self
    where
        T: IntoIterator<Item = (&'a str, bool)>,
    {
        self.order_bys = list
            .into_iter()
            .map(|(name, is_asc)| (name.into(), is_asc))
            .collect();
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

        // Build where
        if let Some(whe) = &self.where_cond {
            parts.push("WHERE".into());
            parts.push(whe.to_string());
        }

        // Build group by
        if !self.group_bys.is_empty() {
            parts.push("GROUP BY".into());
            parts.push(
                self.group_bys
                    .iter()
                    .map(|name| name.clone())
                    .collect::<Vec<_>>()
                    .join(", "),
            )
        }

        // Build order by
        if !self.order_bys.is_empty() {
            parts.push("ORDER BY".into());
            parts.push(
                self.order_bys
                    .iter()
                    .map(|(name, is_asc)| {
                        format!(
                            "{} {}",
                            name,
                            if *is_asc {
                                "ASC".to_string()
                            } else {
                                "DESC".to_string()
                            }
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(", "),
            )
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
