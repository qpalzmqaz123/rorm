use rorm_error::Result;

use crate::Where;

#[derive(Debug, Default)]
pub struct SelectBuilder {
    table: String,
    columns: Vec<String>,
    where_cond: Option<Where>,
    group_bys: Vec<String>,
    order_bys: Vec<(String, bool)>, // (column, is_asc)
    limit: Option<(u64, u64)>,      // (limit, offset)
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
    /// assert_eq!(&a, "SELECT a, b FROM ta");
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
    /// assert_eq!(&a, "SELECT a, b FROM ta");
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
    pub fn group_by<S>(&mut self, col: S) -> &mut Self
    where
        S: ToString,
    {
        self.group_bys.push(col.to_string());
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
    pub fn group_bys<T, S>(&mut self, list: T) -> &mut Self
    where
        T: IntoIterator<Item = S>,
        S: ToString,
    {
        self.group_bys = list.into_iter().map(|v| v.to_string()).collect();
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
    pub fn order_by<S>(&mut self, col: S, is_asc: bool) -> &mut Self
    where
        S: ToString,
    {
        self.order_bys.push((col.to_string(), is_asc));
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
    pub fn order_bys<T, S>(&mut self, list: T) -> &mut Self
    where
        T: IntoIterator<Item = (S, bool)>,
        S: ToString,
    {
        self.order_bys = list
            .into_iter()
            .map(|(name, is_asc)| (name.to_string(), is_asc))
            .collect();
        self
    }

    /// Set limit and offset
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
    ///     .limit(10, 20)
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(&sql, "SELECT a FROM ta WHERE ((a > 1) AND (b < 5)) ORDER BY a ASC, b DESC LIMIT 10 OFFSET 20");
    /// ```
    pub fn limit(&mut self, limit: u64, offset: u64) -> &mut Self {
        self.limit = Some((limit, offset));
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

        // Build limit
        if let Some((limit, offset)) = &self.limit {
            parts.push("LIMIT".into());
            parts.push(limit.to_string());
            parts.push("OFFSET".into());
            parts.push(offset.to_string());
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
