use proc_macro_error::abort;
use quote::ToTokens;
use syn::{Attribute, Data, DataStruct, DeriveInput, Expr, Lit, Meta};

#[derive(Debug, Clone)]
pub struct ColumnInfo {
    pub name: String,
    pub ty: String,
    pub is_auto_increment: bool,
}

#[derive(Debug)]
pub struct TableInfo {
    pub struct_name: String,
    pub table_name: String,
    pub model_name: String,
    pub columns: Vec<ColumnInfo>,
    pub primary_keys: Vec<String>,
}

#[derive(Debug)]
enum AttrInfo {
    TableName(String),
    PrimaryKey,
    AutoIncrement,
}

pub fn parse(input: DeriveInput) -> TableInfo {
    let st = match &input.data {
        Data::Struct(st) => st,
        _ => abort!(input, "Table must be a struct"),
    };

    let struct_name = input.ident.to_string();
    let mut table_name = struct_name.clone();
    let model_name = format!("{}Model", table_name);

    // Parse struct attrs
    for attr in &input.attrs {
        let attr_name = attr.path.to_token_stream().to_string();
        if attr_name != "rorm" {
            abort!(attr, "Attr name must be 'rorm'")
        }

        let attr_infos = parse_rorm_attr(attr);

        // Process attr
        for attr_info in &attr_infos {
            match &attr_info {
                AttrInfo::TableName(name) => table_name = name.clone(),
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
        let mut is_auto_increment = false;

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
                    AttrInfo::AutoIncrement => is_auto_increment = true,
                    _ => abort!(attr, "Invalid column attr field: {:?}", attr_info),
                }
            }
        }

        // Append column
        columns.push(ColumnInfo {
            name,
            ty,
            is_auto_increment,
        });
    }

    (columns, primary_keys)
}

fn parse_rorm_attr(attr: &Attribute) -> Vec<AttrInfo> {
    const PARSE_ERR_STR: &'static str =
        "Parse to metalist failed, syntax is #[rorm(field [= value])]";
    const ARG_HELP: &'static str =
        r#"Syntax is rorm(primary_key | auto_increment | table_name = "NAME", ...)"#;

    let mut attrs = Vec::<AttrInfo>::new();

    // Parse to metalist
    let meta_list = if let Ok(Meta::List(l)) = attr.parse_meta() {
        l
    } else {
        abort!(attr, PARSE_ERR_STR)
    };

    // Parse rorm(field [ = value], ...)
    let call = if let Ok(call) = syn::parse2::<syn::ExprCall>(meta_list.to_token_stream()) {
        call
    } else {
        abort!(attr, PARSE_ERR_STR);
    };

    // Parse args
    for expr in &call.args {
        match expr {
            Expr::Path(p) => {
                let field_name = p.to_token_stream().to_string();
                match field_name.as_str() {
                    // Parse primary_key
                    "primary_key" => attrs.push(AttrInfo::PrimaryKey),

                    // Parse auto_increment
                    "auto_increment" => attrs.push(AttrInfo::AutoIncrement),

                    // Error
                    _ => abort!(expr, "Syntax error while decode path"; help = ARG_HELP),
                }
            }
            Expr::Assign(assign) => {
                let field_name = assign.left.to_token_stream().to_string();
                match field_name.as_str() {
                    // Parse table_name = "NAME"
                    "table_name" => attrs.push(AttrInfo::TableName(get_str(&assign.right))),

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
