use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;

#[proc_macro_derive(Entity, attributes(rorm))]
#[proc_macro_error]
pub fn derive_entity(_item: TokenStream) -> TokenStream {
    TokenStream::new()
}
