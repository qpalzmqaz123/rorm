#[derive(Debug, PartialEq, Eq)]
pub struct TableInfo {
    pub name: &'static str,
    pub columns: &'static [ColumnInfo],
    pub indexes: &'static [IndexInfo],
}

#[derive(Debug, PartialEq, Eq)]
pub struct ColumnInfo {
    pub name: &'static str,
    pub ty: ColumnType,
    pub is_primary_key: bool,                    // Default is false
    pub is_not_null: bool,                       // Default is false
    pub is_auto_increment: bool,                 // Default is false
    pub default: Option<&'static str>,           // Default is None, number is "n", string is "'n'"
    pub is_unique: bool,                         // Default is false
    pub flatten_ref: Option<&'static TableInfo>, // Flatten reference table info
}

// TODO: Add datetime
#[derive(Debug, PartialEq, Eq)]
pub enum ColumnType {
    Bool,
    I8,
    U8,
    I16,
    U16,
    I32,
    U32,
    I64,
    U64,
    F32,
    F64,
    Str(usize),   // String with max length, default is 65536
    Bytes(usize), // Binary data with max length, default is 65536
}

#[derive(Debug, PartialEq, Eq)]
pub struct IndexInfo {
    pub name: &'static str,
    pub keys: &'static [IndexKeyInfo],
}

#[derive(Debug, PartialEq, Eq)]
pub struct IndexKeyInfo {
    pub column_name: &'static str,
}
