use proc_macro_error::abort;
use quote::ToTokens;
use syn::{Attribute, Data, DataStruct, DeriveInput, Expr, Meta};

#[derive(Debug, Clone)]
pub struct ColumnInfo {
    pub name: String,
    pub ty: String,
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
    PrimaryKey,
}

pub fn parse(input: DeriveInput) -> TableInfo {
    let st = match &input.data {
        Data::Struct(st) => st,
        _ => abort!(input, "Table must be a struct"),
    };

    let struct_name = input.ident.to_string();
    let table_name = struct_name.clone();
    let model_name = format!("{}Model", table_name);
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

        // Parse attr
        for attr in &field.attrs {
            let attr_name = attr.path.to_token_stream().to_string();
            if attr_name != "rorm" {
                abort!(attr, "Attr name must be 'rorm'")
            }

            let attrs = parse_rorm_attr(attr);

            // Process attr
            for attr in attrs {
                match attr {
                    AttrInfo::PrimaryKey => primary_keys.push(name.clone()),
                }
            }
        }

        // Append column
        columns.push(ColumnInfo { name, ty });
    }

    (columns, primary_keys)
}

fn parse_rorm_attr(attr: &Attribute) -> Vec<AttrInfo> {
    const PARSE_ERR_STR: &'static str =
        "Parse to metalist failed, syntax is #[rorm(field [= value])]";
    const ARG_HELP: &'static str = "Syntax is rorm([primary_key])";

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
                    _ => abort!(expr, "Syntax error"; help = ARG_HELP),
                }
            }
            _ => abort!(expr, "Syntax error"; help = ARG_HELP),
        }
    }

    attrs
}
