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
    let columns: Vec<&String> = info.columns.iter().map(|v| &v.name).collect();
    let info_toks = gen_table_info(&info);
    let from_row_toks = gen_impl_table_from_row(&info);
    let init_toks = gen_impl_table_init();
    let insert_toks = gen_impl_table_insert(&info);
    let insert_many_toks = gen_impl_table_insert_many(&info);
    let delete_toks = gen_impl_table_delete(&info);
    let update_toks = gen_impl_table_update(&info);
    let find_toks = gen_impl_table_find(&info);
    let find_many_toks = gen_impl_table_find_many(&info);
    let gen_find_sql_and_params_toks = gen_impl_table_gen_find_sql_and_params(&info);

    quote! {
        #[rorm::async_trait]
        impl rorm::Entity for #struct_name {
            type PrimaryKey = #primary_key_type;
            type Model = #model_name;

            const TABLE_NAME: &'static str = #table_name_str;

            const COLUMNS: &'static [&'static str] = &[#(#columns),*];

            const INFO: rorm::TableInfo = #info_toks;

            // fn from_row(row: rorm::pool::Row) -> rorm::error::Result<Self>
            #from_row_toks

            // async fn init(conn: &Connection) -> Result<()>;
            #init_toks

            // async fn insert<M>(conn: &rorm::pool::Connection, model: M) -> rorm::error::Result<#ty>
            // where
            //     M: Into<#model_name>,
            #insert_toks

            // async fn insert_many<T, M>(conn: &rorm::pool::Connection, models: T) -> rorm::error::Result<Vec<#primary_key_type>>
            // where
            //     T: IntoIterator<Item = M>,
            //     M: Into<#model_name>,
            #insert_many_toks

            // async fn delete<M>(conn: &rorm::pool::Connection, model: M) -> rorm::error::Result<()>
            // where
            //     M: Into<#model_name>,
            #delete_toks

            // async fn update<SM, DM>(conn: &rorm::pool::Connection, condition: CM, set: SM) -> rorm::error::Result<()>
            // where
            //     CM: Into<#model_name>,
            //     SM: Into<#model_name>,
            #update_toks

            // async fn find<M>(conn: &rorm::pool::Connection, model: M, option: Option<rorm::FindOption>) -> rorm::error::Result<Self>
            // where
            //    M: Into<#model_name>,
            #find_toks

            // async fn find_many<M>(conn: &rorm::pool::Connection, model: M, option: Option<rorm::FindOption>) -> rorm::error::Result<Vec<Self>>
            // where
            //    M: Into<#model_name>,
            #find_many_toks
        }

        impl #struct_name {
            // fn gen_find_sql_and_params<M>(model: M, option: Option<rorm::FindOption>) -> rorm::error::Result<(String, Vec<rorm::pool::Value>)>
            // where
            //    M: Into<#model_name>,
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
    let gen_set_and_params_field_toks: Vec<TokenStream> = info
        .columns
        .iter()
        .map(|col| {
            let name_str = &col.name;
            let name = str_to_toks(&col.name);
            quote! {
                if let rorm::Set(v) = self.#name {
                    params.push(v.to_value());
                    sets.push(#name_str);
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

            // gen_set_and_params
            pub fn gen_set_and_params(self) -> (Vec<&'static str>, Vec<rorm::pool::Value>) {
                use rorm::pool::ToValue;

                let mut sets = Vec::new();
                let mut params = Vec::new();

                #(#gen_set_and_params_field_toks)*

                (sets, params)
            }
        }
    }
}

fn gen_table_info(info: &TableInfo) -> TokenStream {
    let table_name_str = &info.table_name;
    let columns_toks = info
        .columns
        .iter()
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
        fn from_row(row: rorm::pool::Row) -> rorm::error::Result<Self> {
            Ok(Self {
                #(#field_toks)*
            })
        }
    }
}

fn gen_impl_table_init() -> TokenStream {
    quote! {
        async fn init(conn: &rorm::pool::Connection) -> rorm::error::Result<()> {
            conn.init_table(&Self::INFO).await?;

            Ok(())
        }
    }
}

fn gen_impl_table_insert(info: &TableInfo) -> TokenStream {
    let primary_key_type = gen_primary_key_type_toks(&info.columns, &info.primary_keys);
    let model_name = str_to_toks(&info.model_name);

    quote! {
        async fn insert<M>(conn: &rorm::pool::Connection, model: M) -> rorm::error::Result<#primary_key_type>
        where
            M: Into<#model_name> + Send,
        {
            use rorm::pool::ToValue;

            let model: #model_name = model.into();
            let (cols, params) = model.gen_set_and_params();
            let sql = rorm::query::QueryBuilder::insert(Self::TABLE_NAME)
                .columns(&cols)
                .values(cols.iter().map(|_| "?".into()).collect::<Vec<_>>())
                .build()?;
            let key = conn
                .execute_one(&sql, params)
                .await?;

            // FIXME: Union index is not currently supported
            Ok(key as #primary_key_type)
        }
    }
}

fn gen_impl_table_insert_many(info: &TableInfo) -> TokenStream {
    let primary_key_type = gen_primary_key_type_toks(&info.columns, &info.primary_keys);
    let model_name = str_to_toks(&info.model_name);

    quote! {
        async fn insert_many<T, M>(conn: &rorm::pool::Connection, models: T) -> rorm::error::Result<Vec<#primary_key_type>>
        where
            T: IntoIterator<Item = M> + Send,
            M: Into<#model_name> + Send,
        {
            use rorm::pool::ToValue;

            let mut sql = Option::<String>::None;
            let mut first_cols = Option::<Vec<&'static str>>::None;
            let mut params_list = Vec::new();

            for model in models.into_iter() {
                let model: #model_name = model.into();
                let (cols, params) = model.gen_set_and_params();

                // Generate sql
                if sql.is_none() {
                    sql = Some(rorm::query::QueryBuilder::insert(Self::TABLE_NAME)
                        .columns(&cols)
                        .values(cols.iter().map(|_| "?".into()).collect::<Vec<_>>())
                        .build()?);
                }

                // Check model cols
                if let Some(first_cols) = &first_cols {
                    if first_cols != &cols {
                        return Err(rorm::error::argument!("Model insert rows mismatch, first_cols: {:?}, recevied: {:?}", first_cols, cols));
                    }
                } else {
                    first_cols = Some(cols);
                }

                // Append params
                params_list.push(params);
            }

            if let Some(sql) = sql {
                let keys = conn.execute_many(&sql, params_list).await?;
                Ok(keys.into_iter().map(|k| k as #primary_key_type).collect())
            } else {
                Ok(Vec::new())
            }
        }
    }
}

fn gen_impl_table_delete(info: &TableInfo) -> TokenStream {
    let model_name = str_to_toks(&info.model_name);

    quote! {
        async fn delete<M>(conn: &rorm::pool::Connection, model: M) -> rorm::error::Result<()>
        where
            M: Into<#model_name> + Send,
        {
            let model: #model_name = model.into();
            let mut sql_builder = rorm::query::QueryBuilder::delete(Self::TABLE_NAME);
            let (cond, params) = model.gen_where_and_params();

            // Set builder
            if let Some(cond) = cond {
                sql_builder.where_cond(cond);
            }

            // Build sql
            let sql = sql_builder.build()?;

            // Execute
            conn.execute_one(&sql, params).await?;

            Ok(())
        }
    }
}

fn gen_impl_table_update(info: &TableInfo) -> TokenStream {
    let model_name = str_to_toks(&info.model_name);

    quote! {
        async fn update<CM, SM>(conn: &rorm::pool::Connection, condition: CM, set: SM) -> rorm::error::Result<()>
        where
            CM: Into<#model_name> + Send,
            SM: Into<#model_name> + Send,
        {
            let condition: #model_name = condition.into();
            let set: #model_name = set.into();
            let mut sql_builder = rorm::query::QueryBuilder::update(Self::TABLE_NAME);
            let (cond, mut cond_params) = condition.gen_where_and_params();
            let (set_cols, set_params) = set.gen_set_and_params();

            // Set builder
            if let Some(cond) = cond {
                sql_builder.where_cond(cond);
            }

            for set_col in set_cols {
                sql_builder.set(set_col, "?".into());
            }

            // Build sql
            let sql = sql_builder.build()?;

            let mut params = set_params;
            params.append(&mut cond_params);

            // Execute
            conn.execute_one(&sql, params).await?;

            Ok(())
        }
    }
}

fn gen_impl_table_find(info: &TableInfo) -> TokenStream {
    let model_name = str_to_toks(&info.model_name);

    quote! {
        async fn find<M>(conn: &rorm::pool::Connection, model: M, option: Option<rorm::FindOption>) -> rorm::error::Result<Self>
        where
            M: Into<#model_name> + Send,
        {
            let (sql, params) = Self::gen_find_sql_and_params(model, option)?;

            // Query
            let res = conn
                .query_one_map(&sql, params, |row| Self::from_row(row))
                .await?;

            Ok(res)
        }
    }
}

fn gen_impl_table_find_many(info: &TableInfo) -> TokenStream {
    let model_name = str_to_toks(&info.model_name);

    quote! {
        async fn find_many<M>(conn: &rorm::pool::Connection, model: M, option: Option<rorm::FindOption>) -> rorm::error::Result<Vec<Self>>
        where
            M: Into<#model_name> + Send,
        {
            let (sql, params) = Self::gen_find_sql_and_params(model, option)?;

            // Query
            let res_list = conn
                .query_many_map(&sql, params, |row| Self::from_row(row))
                .await?;

            Ok(res_list)
        }
    }
}

fn gen_impl_table_gen_find_sql_and_params(info: &TableInfo) -> TokenStream {
    let model_name = str_to_toks(&info.model_name);

    quote! {
        fn gen_find_sql_and_params<M>(model: M, option: Option<rorm::FindOption>) -> rorm::error::Result<(String, Vec<rorm::pool::Value>)>
        where
            M: Into<#model_name>,
        {
            let model: #model_name = model.into();
            let mut sql_builder = rorm::query::QueryBuilder::select(Self::TABLE_NAME);
            let (cond, params) = model.gen_where_and_params();
            let already_has_cond = cond.is_some();

            // Set builder
            sql_builder.columns(Self::COLUMNS);
            if let Some(cond) = cond {
                sql_builder.where_cond(cond);
            }

            // Set option
            if let Some(option) = option {
                // Check model condition
                if already_has_cond {
                    return Err(rorm::error::argument!("Where condition conflict, model must be empty when option.where_cond was set; model: {}", std::any::type_name::<#model_name>()));
                }

                // Update builder
                option.update_sql_builder(&mut sql_builder);
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
