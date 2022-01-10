use std::{collections::HashMap, str::FromStr};

use proc_macro2::TokenStream;
use quote::quote;

use crate::{ColumnInfo, TableInfo};

pub fn generate(info: TableInfo) -> TokenStream {
    let impl_table_toks = gen_impl_table(&info);
    let model_toks = gen_model(&info);

    quote! {
        #impl_table_toks

        #model_toks
    }
}

fn gen_impl_table(info: &TableInfo) -> TokenStream {
    let primary_key_type = gen_primary_key_type_toks(&info.columns, &info.primary_keys);
    let table_name_str = &info.table_name;
    let model_name = str_to_toks(&info.model_name);
    let struct_name = str_to_toks(&info.struct_name);
    let columns: Vec<&String> = info
        .columns
        .iter()
        .filter(|col| col.relation.is_none()) // Skip relation fields
        .map(|v| &v.name)
        .collect();
    let info_toks = gen_table_info(&info);
    let from_row_toks = gen_impl_table_from_row(&info);

    quote! {
        #[rorm::async_trait]
        impl rorm::Entity for #struct_name {
            type PrimaryKey = #primary_key_type;
            type Model = #model_name;

            const TABLE_NAME: &'static str = #table_name_str;

            const COLUMNS: &'static [&'static str] = &[#(#columns),*];

            const INFO: rorm::TableInfo = #info_toks;

            // async fn from_row(conn: &Connection, row: Row) -> Result<Self>;
            #from_row_toks
        }
    }
}

fn gen_model(info: &TableInfo) -> TokenStream {
    let struct_name = str_to_toks(&info.struct_name);
    let model_name = str_to_toks(&info.model_name);
    let field_toks: Vec<TokenStream> = info
        .columns
        .iter()
        .filter(|col| col.relation.is_none()) // Skip relation fields
        .map(|col| {
            let name = str_to_toks(&col.name);
            let ty = str_to_toks(&col.ty);
            quote! {
                pub #name: rorm::ModelColumn<#ty>,
            }
        })
        .collect();
    let from_field_toks: Vec<TokenStream> = info
        .columns
        .iter()
        .filter(|col| col.relation.is_none()) // Skip relation fields
        .map(|col| {
            let name = str_to_toks(&col.name);
            quote! {
                #name: rorm::Set(v.#name),
            }
        })
        .collect();
    let into_set_pairs_field_toks: Vec<TokenStream> = info
        .columns
        .iter()
        .filter(|col| col.relation.is_none()) // Skip relation field
        .map(|col| {
            let name_str = &col.name;
            let name = str_to_toks(&col.name);

            if col.is_serde_json {
                quote! {
                    if let rorm::Set(v) = self.#name {
                        match serde_json::to_string(&v) {
                            Ok(s) => arr.push((#name_str, s.to_value())),
                            Err(e) => eprintln!("ERROR: Parse '{:?}' to json error: {}", v, e), // FIXME: catch error
                        }
                    }
                }
            } else {
                quote! {
                    if let rorm::Set(v) = self.#name {
                        arr.push((#name_str, v.to_value()));
                    }
                }
            }
        })
        .collect();
    let primary_key_types = gen_primary_key_type_toks(&info.columns, &info.primary_keys);
    let from_primary_key_field_toks = match info.primary_keys.len() {
        0 => Vec::new(),
        1 => {
            let name = str_to_toks(&info.primary_keys[0]);
            vec![quote! {
                #name: rorm::Set(v),
            }]
        }
        _ => info
            .primary_keys
            .iter()
            .enumerate()
            .map(|(index, name)| {
                let name = str_to_toks(name);
                quote! {
                    #name: rorm::Set(v.#index),
                }
            })
            .collect(),
    };

    quote! {
        // Model name
        #[derive(Debug, Default)]
        struct #model_name {
            #(#field_toks)*
        }

        // Impl from table to model
        impl From<#struct_name> for #model_name {
            fn from(v: #struct_name) -> Self {
                #model_name {
                    #(#from_field_toks)*
                }
            }
        }

        // Impl from primary key's type to model
        impl From<#primary_key_types> for #model_name {
            fn from(v: #primary_key_types) -> Self {
                #model_name {
                    #(#from_primary_key_field_toks)*
                    ..Default::default()
                }
            }
        }

        // Impl trait
        impl rorm::Model<#primary_key_types> for #model_name {
            fn into_set_pairs(self) -> Vec<(&'static str, rorm::Value)> {
                use rorm::ToValue;

                let mut arr = vec![];

                #(#into_set_pairs_field_toks)*

                arr
            }

            fn to_primary_key(id: u64) -> #primary_key_types {
                id as #primary_key_types
            }
        }
    }
}

fn gen_table_info(info: &TableInfo) -> TokenStream {
    let table_name_str = &info.table_name;
    let columns_toks = info
        .columns
        .iter()
        .filter(|col| col.relation.is_none()) // Skip relation fields
        .map(|col| {
            let name = &col.name;
            let (ty_toks, is_not_null) = gen_column_type_and_is_not_null(&col);
            let is_primary_key = info.primary_keys.contains(name);
            let is_auto_increment = col.is_auto_increment;
            let default = if let Some(def) = &col.default {
                quote! { Some(#def) }
            } else {
                quote! { None }
            };
            let is_unique = col.is_unique;

            quote! {
                rorm::ColumnInfo {
                    name: #name,
                    ty: #ty_toks,
                    is_primary_key: #is_primary_key,
                    is_not_null: #is_not_null,
                    is_auto_increment: #is_auto_increment,
                    default: #default,
                    is_unique: #is_unique,
                }
            }
        })
        .collect::<Vec<_>>();
    let index_toks = info
        .indexes
        .iter()
        .map(|index| {
            let index_name = format!("{}_index_{}", info.table_name, index.join("_"));
            let keys_toks = index
                .iter()
                .map(|col| {
                    let column_name = col;
                    quote! {
                        rorm::IndexKeyInfo {
                            column_name: #column_name,
                        }
                    }
                })
                .collect::<Vec<_>>();

            quote! {
                rorm::IndexInfo {
                    name: #index_name,
                    keys: &[#(#keys_toks),*],
                }
            }
        })
        .collect::<Vec<_>>();

    quote! {
        rorm::TableInfo {
            name: #table_name_str,
            columns: &[#(#columns_toks),*],
            indexes: &[#(#index_toks),*],
        }
    }
}

fn gen_impl_table_from_row(info: &TableInfo) -> TokenStream {
    let mut index = 0usize; // The relation field will occupy the index, so the index needs to be calculated separately
    let field_toks: Vec<TokenStream> = info
        .columns
        .iter()
        .map(|col| {
            let name = str_to_toks(&col.name);

            if let Some(relation) = &col.relation {
                // Relation field
                let relation_struct = str_to_toks(&relation.ty);
                let relation_model = str_to_toks(&format!("{}Model", relation.ty)); // FIXME: Model rule must be defined in one place
                let relation_field = str_to_toks(&relation.ref_col);
                let relation_self_field_index = info.columns.iter().filter(|v| v.relation.is_none()).position(|v| v.name == relation.self_col).expect(&format!("Relation self_col '{}' not found in table '{}'", relation.self_col, info.struct_name));
                let model_toks = quote! {
                    #relation_model {
                        #relation_field: rorm::Set(row.get(#relation_self_field_index)?),
                        ..Default::default()
                    }
                };

                if relation.is_vec {
                    // Relation is vec
                    quote! {
                        #name: #relation_struct::find().filter_model(#model_toks).execute(conn).await?,
                    }
                } else {
                    if relation.is_not_null {
                        // Relation is normal type
                        quote! {
                            #name: #relation_struct::find().filter_model(#model_toks).execute(conn).await?.into_iter().next()
                                .ok_or(rorm::error::database!("Relation {}-{} > {}-{} fond empty rows", std::any::type_name::<Self>(), stringify!(#name), std::any::type_name::<#relation_struct>(), stringify!(#relation_field)))?,
                        }
                    } else {
                        // Relation is option
                        quote! {
                            #name: #relation_struct::find().filter_model(#model_toks).execute(conn).await?.into_iter().next(),
                        }
                    }
                }
            } else {
                // Normal field
                let toks = if col.is_serde_json {
                    // Parse from json
                    quote! {
                        #name: {
                            let s = row.get::<String>(#index)?;
                            serde_json::from_str(&s).map_err(|e| rorm::error::from_value!("Convert json '{}' to {}.{} failed", s, std::any::type_name::<Self>(), stringify!(#name)))?
                        }
                    }
                } else {
                    // Not json
                    quote! {
                        #name: row.get(#index)?,
                    }
                };

                // Increase index
                index += 1;

                toks
            }
        })
        .collect();

    quote! {
        async fn from_row(conn: &rorm::Connection, row: rorm::Row) -> rorm::error::Result<Self> {
            Ok(Self {
                #(#field_toks)*
            })
        }
    }
}

fn str_to_toks(s: &str) -> TokenStream {
    TokenStream::from_str(s).unwrap()
}

fn gen_primary_key_type_toks(columns: &[ColumnInfo], primary_keys: &[String]) -> TokenStream {
    // Generate type map for lookup
    let column_type_map = columns
        .iter()
        .fold(HashMap::<&str, &str>::new(), |mut map, info| {
            map.insert(info.name.as_str(), info.ty.as_str());
            map
        });

    // Generate primary key type array
    let primary_key_types = primary_keys
        .iter()
        .map(|k| column_type_map[k.as_str()])
        .collect::<Vec<&str>>();

    match primary_key_types.len() {
        // No parmary key, type is empty
        0 => quote! {()},

        // Has only one primary key, type is key's type
        1 => {
            let ty = str_to_toks(primary_key_types[0]);
            quote! {#ty}
        }

        // More than one primary key, type is tuple
        _ => {
            let types = primary_key_types
                .iter()
                .map(|ty| str_to_toks(ty))
                .collect::<Vec<_>>();
            quote! {(#(#types),*)}
        }
    }
}

fn gen_column_type_and_is_not_null(col: &ColumnInfo) -> (TokenStream, bool) {
    let length = col.length.unwrap_or(65535);

    match col.sql_ty.replace(" ", "").as_str() {
        "bool" => (quote! { rorm::ColumnType::Bool }, true),
        "i8" => (quote! { rorm::ColumnType::I8 }, true),
        "u8" => (quote! { rorm::ColumnType::U8 }, true),
        "i16" => (quote! { rorm::ColumnType::I16 }, true),
        "u16" => (quote! { rorm::ColumnType::U16 }, true),
        "i32" => (quote! { rorm::ColumnType::I32 }, true),
        "u32" => (quote! { rorm::ColumnType::U32 }, true),
        "i64" => (quote! { rorm::ColumnType::I64 }, true),
        "u64" => (quote! { rorm::ColumnType::U64 }, true),
        "f32" => (quote! { rorm::ColumnType::F32 }, true),
        "f64" => (quote! { rorm::ColumnType::F64 }, true),
        "String" => (quote! { rorm::ColumnType::Str(#length) }, true),
        "Vec<u8>" => (quote! { rorm::ColumnType::Bytes(#length) }, true),
        "Option<bool>" => (quote! { rorm::ColumnType::Bool }, false),
        "Option<i8>" => (quote! { rorm::ColumnType::I8 }, false),
        "Option<u8>" => (quote! { rorm::ColumnType::U8 }, false),
        "Option<i16>" => (quote! { rorm::ColumnType::I16 }, false),
        "Option<u16>" => (quote! { rorm::ColumnType::U16 }, false),
        "Option<i32>" => (quote! { rorm::ColumnType::I32 }, false),
        "Option<u32>" => (quote! { rorm::ColumnType::U32 }, false),
        "Option<i64>" => (quote! { rorm::ColumnType::I64 }, false),
        "Option<u64>" => (quote! { rorm::ColumnType::U64 }, false),
        "Option<f32>" => (quote! { rorm::ColumnType::F32 }, false),
        "Option<f64>" => (quote! { rorm::ColumnType::F64 }, false),
        "Option<String>" => (quote! { rorm::ColumnType::Str(#length) }, false),
        "Option<Vec<u8>>" => (quote! { rorm::ColumnType::Bytes(#length) }, false),
        _ => panic!("Unsupported column type '{}', name: '{}'", col.ty, col.name),
    }
}
