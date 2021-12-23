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
    let table_name_str = &info.table_name;
    let struct_name = str_to_toks(&info.struct_name);
    let columns: Vec<&String> = info.columns.iter().map(|v| &v.name).collect();
    let from_row_toks = gen_impl_table_from_row(&info);
    let insert_toks = gen_impl_table_insert(&info);
    let find_one_toks = gen_impl_table_find_one(&info);
    let find_toks = gen_impl_table_find(&info);
    let find_by_primary_key_toks = gen_impl_table_find_by_primary_key(&info);
    let gen_find_sql_and_params_toks = gen_impl_table_gen_find_sql_and_params(&info);

    quote! {
        impl #struct_name {
            /*
             * Const fields
             */

            pub const TABLE_NAME: &'static str = #table_name_str;
            pub const COLUMNS: &'static [&'static str] = &[#(#columns),*];

            /*
             * Public methods
             */

            // pub fn from_row(row: rorm::pool::Row) -> rorm::error::Result<Self>
            #from_row_toks

            // pub async fn insert<M>(model: M, conn: &rorm::pool::Connection) -> rorm::error::Result<#ty>
            #insert_toks

            // pub async fn find_one<M>(model: M, conn: &rorm::pool::Connection) -> rorm::error::Result<Self>
            #find_one_toks

            // pub async fn find<M>(model: M, conn: &rorm::pool::Connection) -> rorm::error::Result<Vec<Self>>
            #find_toks

            // pub async fn find_by_#primary_key(key: u32, conn: &rorm::pool::Connection) -> rorm::error::Result<Self>
            #find_by_primary_key_toks

            /*
             * Private methods
             */

            // fn gen_find_sql_and_params<M>(model: M) -> rorm::error::Result<(String, Vec<rorm::pool::Value>)>
            #gen_find_sql_and_params_toks
        }
    }
}

fn gen_model(info: &TableInfo) -> TokenStream {
    let struct_name = str_to_toks(&info.struct_name);
    let model_name = str_to_toks(&info.model_name);
    let field_toks: Vec<TokenStream> = info
        .columns
        .iter()
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
        .map(|col| {
            let name = str_to_toks(&col.name);
            quote! {
                #name: rorm::Set(v.#name),
            }
        })
        .collect();
    let gen_where_and_params_field_toks: Vec<TokenStream> = info
        .columns
        .iter()
        .map(|col| {
            let name_str = &col.name;
            let name = str_to_toks(&col.name);
            quote! {
                if let rorm::Set(v) = self.#name {
                    let c = rorm::query::eq!(#name_str, "?");
                    cond = if let Some(cond) = cond {
                        Some(rorm::query::and!(cond, c))
                    } else {
                        Some(c)
                    };

                    params.push(v.to_value());
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
        #[derive(Debug, Clone, Default)]
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

        // Impl model
        impl #model_name {
            // gen_where_and_params
            pub fn gen_where_and_params(self) -> (Option<rorm::query::Where>, Vec<rorm::pool::Value>) {
                use rorm::pool::ToValue;

                let mut params = Vec::new();
                let mut cond = None;

                #(#gen_where_and_params_field_toks)*

                (cond, params)
            }
        }
    }
}

fn gen_impl_table_from_row(info: &TableInfo) -> TokenStream {
    let field_toks: Vec<TokenStream> = info
        .columns
        .iter()
        .enumerate()
        .map(|(index, col)| {
            let name = str_to_toks(&col.name);
            quote! {
                #name: row.get(#index)?,
            }
        })
        .collect();

    quote! {
        pub fn from_row(row: rorm::pool::Row) -> rorm::error::Result<Self> {
            Ok(Self {
                #(#field_toks)*
            })
        }
    }
}

fn gen_impl_table_insert(info: &TableInfo) -> TokenStream {
    let primary_key_type = gen_primary_key_type_toks(&info.columns, &info.primary_keys);
    let model_name = str_to_toks(&info.model_name);
    let sql_params_toks: Vec<TokenStream> = info
        .columns
        .iter()
        .map(|col| {
            let name = str_to_toks(&col.name);
            quote! {
                model.#name.to_value()
            }
        })
        .collect();
    let values_toks = (0..info.columns.len())
        .map(|_| quote! {"?".into()})
        .collect::<Vec<_>>();

    quote! {
        pub async fn insert<M>(model: M, conn: &rorm::pool::Connection) -> rorm::error::Result<#primary_key_type>
        where
            M: Into<#model_name>,
        {
            use rorm::pool::ToValue;

            let model: #model_name = model.into();
            let sql = rorm::query::QueryBuilder::insert(Self::TABLE_NAME)
                .columns(&Self::COLUMNS)
                .values([#(#values_toks),*])
                .build()?;
            let key = conn
                .execute_one(&sql, vec![#(#sql_params_toks),*])
                .await?;

            // FIXME: Union index is not currently supported
            Ok(key as #primary_key_type)
        }
    }
}

fn gen_impl_table_find_one(info: &TableInfo) -> TokenStream {
    let model_name = str_to_toks(&info.model_name);

    quote! {
        pub async fn find_one<M>(model: M, conn: &rorm::pool::Connection) -> rorm::error::Result<Self>
        where
            M: Into<#model_name>,
        {
            let (sql, params) = Self::gen_find_sql_and_params(model)?;

            // Query
            let res = conn
                .query_one_map(&sql, params, |row| Self::from_row(row))
                .await?;

            Ok(res)
        }
    }
}

fn gen_impl_table_find(info: &TableInfo) -> TokenStream {
    let model_name = str_to_toks(&info.model_name);

    quote! {
        pub async fn find<M>(model: M, conn: &rorm::pool::Connection) -> rorm::error::Result<Vec<Self>>
        where
            M: Into<#model_name>,
        {
            let (sql, params) = Self::gen_find_sql_and_params(model)?;

            // Query
            let res_list = conn
                .query_many_map(&sql, params, |row| Self::from_row(row))
                .await?;

            Ok(res_list)
        }
    }
}

fn gen_impl_table_find_by_primary_key(info: &TableInfo) -> TokenStream {
    let func_name = str_to_toks(&format!("find_by_{}", info.primary_keys.join("_")));
    let primary_keys_type = gen_primary_key_type_toks(&info.columns, &info.primary_keys);

    let fields_toks = match info.primary_keys.len() {
        0 => Vec::new(),
        1 => {
            let key = str_to_toks(&info.primary_keys[0]);
            vec![quote! {
                #key: rorm::Set(key),
            }]
        }
        _ => info
            .primary_keys
            .iter()
            .enumerate()
            .map(|(index, key)| {
                let key = str_to_toks(key);
                quote! {
                    #key: rorm::Set(key.#index),
                }
            })
            .collect(),
    };

    quote! {
        pub async fn #func_name(key: #primary_keys_type, conn: &rorm::pool::Connection) -> rorm::error::Result<Self> {
            let model = UserModel {
                #(#fields_toks)*
                ..Default::default()
            };

            Ok(Self::find_one(model, conn).await?)
        }
    }
}

fn gen_impl_table_gen_find_sql_and_params(info: &TableInfo) -> TokenStream {
    let model_name = str_to_toks(&info.model_name);

    quote! {
        fn gen_find_sql_and_params<M>(model: M) -> rorm::error::Result<(String, Vec<rorm::pool::Value>)>
        where
            M: Into<#model_name>,
        {
            let model: #model_name = model.into();
            let mut sql_builder = rorm::query::QueryBuilder::select(Self::TABLE_NAME);
            let (cond, params) = model.gen_where_and_params();

            // Set builder
            sql_builder.columns(&Self::COLUMNS);
            if let Some(cond) = cond {
                sql_builder.where_cond(cond);
            }

            // Build sql
            let sql = sql_builder.build()?;

            Ok((sql, params))
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
