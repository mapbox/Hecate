extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};
use super::config::AuthParse;

#[proc_macro_derive(AuthParse)]
pub fn auth_parse(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;

    let expanded = quote! {
        impl AuthParse for #name {
            fn parse(&serde_json::Value) -> Result<Box<Self>, HecateError> {
                Err(HecateError::new(400, String::from("test"), None))
            }
        }
    }
}
