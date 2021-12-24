use crate::query::{SelectBuilder, Where};

#[derive(Debug, Default)]
pub struct FindOption {
    pub where_cond: Option<Where>,
    pub groups: Vec<String>,         // GROUP BY col1, col2, ...
    pub orders: Vec<(String, bool)>, // ORDER BY col1 is_asc1, col2 is_asc2, ...
    pub limit: Option<(u64, u64)>,   // (Limit, offset)
}

impl FindOption {
    pub fn update_sql_builder(self, builder: &mut SelectBuilder) {
        // Process where
        if let Some(cond) = self.where_cond {
            builder.where_cond(cond);
        }

        // Process group by
        builder.group_bys(self.groups);

        // Process order by
        builder.order_bys(self.orders);

        // Process limit
        if let Some(lim) = self.limit {
            builder.limit(lim.0, lim.1);
        }
    }
}
