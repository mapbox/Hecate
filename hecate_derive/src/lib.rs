extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields};

#[proc_macro_derive(AuthParse, attributes(default))]
pub fn auth_parse(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let nameq = format!("{}", name);

    let mut some = vec![];
    let mut none = vec![];

    match input.data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => {
                for fnamed in &fields.named {
                    let fname = match &fnamed.ident {
                        Some(fname) => fname,
                        None => unimplemented!()
                    };

                    let fnameq = format!("{}", fname);

                    some.push(quote! {
                        #fname: get_kv(#nameq, #fnameq, value)?,
                    });

                    none.push(quote! {
                        #fname: String::from("disabled"),
                    });
                }
            },
            _ => unimplemented!()

        },
        _ => unimplemented!(),
    }

    let expanded = quote! {
        impl AuthParse for #name {
            fn parse(value: Option<&serde_json::Value>) -> Result<Box<Self>, HecateError> {
                match value {
                    Some(ref value) => {
                        Ok(Box::new(#name {
                            #(#some)*
                        }))
                    },
                    None => {
                        Ok(Box::new(#name {
                            #(#none)*
                        }))
                    }
                }
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}
