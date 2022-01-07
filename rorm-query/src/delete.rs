use rorm_error::Result;

use crate::{Filter, Where};

#[derive(Debug, Default)]
pub struct DeleteBuilder {
    table: String,
    filter: Filter,
}

impl DeleteBuilder {
    pub fn new<S>(table: S) -> Self
    where
        S: ToString,
    {
        Self {
            table: table.to_string(),
            ..Default::default()
        }
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

        // Build filter
        parts.push(self.filter.build()?);

        Ok(parts.join(" "))
    }

    /// Validate builder
    fn validate(&self) -> Result<()> {
        Ok(())
    }
}

crate::lazy_impl_filer_for_struct! { DeleteBuilder }
