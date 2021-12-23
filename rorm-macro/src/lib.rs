mod generator;
mod parser;

use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use syn::{parse_macro_input, DeriveInput};

use parser::{ColumnInfo, TableInfo};

#[proc_macro_derive(Entity, attributes(rorm))]
#[proc_macro_error]
pub fn derive_entity(item: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(item);
    let info = parser::parse(input);
    let stream = generator::generate(info);

    stream.into()
}
