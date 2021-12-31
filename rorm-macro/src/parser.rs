use proc_macro_error::abort;
use quote::ToTokens;
use syn::{Attribute, Data, DataStruct, DeriveInput, Expr, Lit};

#[derive(Debug, Clone)]
pub struct ColumnInfo {
    pub name: String,   // Column rust name
    pub ty: String,     // Column rust type
    pub sql_ty: String, // User specific type, use for generate sql type, default it's same with ty
    pub length: Option<usize>,
    pub is_auto_increment: bool,
    pub default: Option<String>, // Sql literal
    pub is_unique: bool,
}

#[derive(Debug)]
pub struct TableInfo {
    pub struct_name: String,
    pub table_name: String,
    pub model_name: String,
    pub columns: Vec<ColumnInfo>,
    pub primary_keys: Vec<String>,
    pub indexes: Vec<Vec<String>>,
}

#[derive(Debug)]
enum AttrInfo {
    TableName(String),
    PrimaryKey,
    Length(usize),
    AutoIncrement,
    Type(String),
    Index(Vec<String>),
    Default(String),
    Unique,
}

pub fn parse(input: DeriveInput) -> TableInfo {
    let st = match &input.data {
        Data::Struct(st) => st,
        _ => abort!(input, "Table must be a struct"),
    };

    let struct_name = input.ident.to_string();
    let mut table_name = struct_name.clone();
    let model_name = format!("{}Model", table_name);
    let mut indexes = Vec::<Vec<String>>::new();

    // Parse struct attrs
    for attr in &input.attrs {
        let attr_name = attr.path.to_token_stream().to_string();
        if attr_name != "rorm" {
            abort!(attr, "Attr name must be 'rorm'")
        }

        let attr_infos = parse_rorm_attr(attr);

        // Process attr
        for attr_info in attr_infos {
            match attr_info {
                AttrInfo::TableName(name) => table_name = name,
                AttrInfo::Index(cols) => {
                    if cols.len() == 0 {
                        abort!(attr, "Empty columns");
                    }

                    indexes.push(cols);
                }
                _ => abort!(attr, "Invalid struct attr field: {:?}", attr_info),
            }
        }
    }

    // Parse columns
    let (columns, primary_keys) = parse_columns(st);

    TableInfo {
        struct_name,
        table_name,
        model_name,
        columns,
        primary_keys,
        indexes,
    }
}

fn parse_columns(st: &DataStruct) -> (Vec<ColumnInfo>, Vec<String>) {
    let mut columns = Vec::<ColumnInfo>::new();
    let mut primary_keys = Vec::<String>::new();

    for field in &st.fields {
        // Generate name and type
        let name = if let Some(name) = &field.ident {
            name.to_string()
        } else {
            abort!(field, "Field must be named");
        };
        let ty = field.ty.to_token_stream().to_string();
        let mut sql_ty = ty.clone();
        let mut length = None;
        let mut is_auto_increment = false;
        let mut default = Option::<String>::None;
        let mut is_unique = false;

        // Parse attr
        for attr in &field.attrs {
            let attr_name = attr.path.to_token_stream().to_string();
            if attr_name != "rorm" {
                abort!(attr, "Attr name must be 'rorm'")
            }

            let attr_infos = parse_rorm_attr(attr);

            // Process attr
            for attr_info in attr_infos {
                match attr_info {
                    AttrInfo::PrimaryKey => primary_keys.push(name.clone()),
                    AttrInfo::Length(len) => length = Some(len),
                    AttrInfo::AutoIncrement => is_auto_increment = true,
                    AttrInfo::Type(ty) => sql_ty = ty,
                    AttrInfo::Default(def) => default = Some(def),
                    AttrInfo::Unique => is_unique = true,
                    _ => abort!(attr, "Invalid column attr field: {:?}", attr_info),
                }
            }
        }

        // Append column
        columns.push(ColumnInfo {
            name,
            ty,
            sql_ty,
            length,
            is_auto_increment,
            default,
            is_unique,
        });
    }

    (columns, primary_keys)
}

fn parse_rorm_attr(attr: &Attribute) -> Vec<AttrInfo> {
    const PARSE_ERR_STR: &'static str = "Parse failed, syntax is #[rorm(field [= value])]";
    const ARG_HELP: &'static str = r#"Syntax is rorm(primary_key | auto_increment | unique | table_name = "NAME" | sql_type = RUST_TYPE | length = NUMBER | default = (NUMBER | STR) | index = [col1, col2, ...], ...)"#;

    let mut attrs = Vec::<AttrInfo>::new();

    // Generate function call tokens: rorm(xxx)
    let path = attr.path.clone();
    let toks = attr.tokens.clone();
    let call_toks = quote::quote! {#path #toks};

    let args = if let Ok(call) = syn::parse2::<syn::ExprCall>(call_toks) {
        call.args
    } else {
        abort!(attr.tokens, PARSE_ERR_STR);
    };

    // Parse args
    for expr in &args {
        match expr {
            Expr::Path(p) => {
                let field_name = p.to_token_stream().to_string();
                match field_name.as_str() {
                    // Parse primary_key
                    "primary_key" => attrs.push(AttrInfo::PrimaryKey),

                    // Parse auto_increment
                    "auto_increment" => attrs.push(AttrInfo::AutoIncrement),

                    // Parse auto_increment
                    "unique" => attrs.push(AttrInfo::Unique),

                    // Error
                    _ => abort!(expr, "Syntax error while decode path"; help = ARG_HELP),
                }
            }
            Expr::Assign(assign) => {
                let field_name = assign.left.to_token_stream().to_string();
                match field_name.as_str() {
                    // Parse table_name = "NAME"
                    "table_name" => attrs.push(AttrInfo::TableName(get_str(&assign.right))),

                    // Parse length = NUMBER
                    "length" => attrs.push(AttrInfo::Length(get_num(&assign.right))),

                    // Parse type = RUST_TYPE
                    "sql_type" => attrs.push(AttrInfo::Type(get_path(&assign.right))),

                    // Parse index = [col1, col2, ...]
                    "index" => attrs.push(AttrInfo::Index(get_path_arr(&assign.right))),

                    // Parse default = (NUMBER | STR)
                    "default" => attrs.push(AttrInfo::Default(get_sql_lit(&assign.right))),

                    // Error
                    _ => abort!(expr, "Syntax error while decode assign"; help = ARG_HELP),
                }
            }
            _ => abort!(expr, "Syntax error while metch expr"; help = ARG_HELP),
        }
    }

    attrs
}

/// Get string from expr
fn get_str(expr: &Expr) -> String {
    if let Expr::Lit(lit) = expr {
        if let Lit::Str(s) = &lit.lit {
            return s.value();
        }
    }

    abort!(expr, "Expect string")
}

/// Get number from expr
fn get_num(expr: &Expr) -> usize {
    if let Expr::Lit(lit) = expr {
        if let Lit::Int(n) = &lit.lit {
            if let Ok(n) = n.base10_parse::<usize>() {
                return n;
            }
        }
    }

    abort!(expr, "Expect integer")
}

/// Get type from expr
fn get_path(expr: &Expr) -> String {
    if let Expr::Path(path) = expr {
        return path.path.segments.last().unwrap().ident.to_string();
    }

    abort!(expr, "Expect path")
}

/// Get path array from expr
fn get_path_arr(expr: &Expr) -> Vec<String> {
    if let Expr::Array(arr) = expr {
        return arr.elems.iter().map(|exp| get_path(exp)).collect();
    }

    abort!(expr, "Expect path array")
}

/// Get sql lit from expr
/// for example: 1 => "1", 1.1 => "1.1", "str" => "'str'"
fn get_sql_lit(expr: &Expr) -> String {
    if let Expr::Lit(lit) = expr {
        match &lit.lit {
            Lit::Int(n) => {
                return n.to_string();
            }
            Lit::Float(n) => {
                return n.to_string();
            }
            Lit::Str(s) => {
                return format!("'{}'", s.value());
            }
            _ => {}
        }
    }

    abort!(expr, "Expect literal")
}
