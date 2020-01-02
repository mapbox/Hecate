extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields};

#[proc_macro_derive(AuthModule, attributes(default, valid))]
pub fn auth_parse(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let parse = parse_fn(&input);
    let default = default_fn(&input);
    let valid = valid_fn(&input);

    let name = &input.ident;

    let expanded = quote! {
        impl AuthModule for #name {
            #parse
            #default
            #valid
        }
    };

    proc_macro::TokenStream::from(expanded)
}

fn valid_fn(input: &DeriveInput) -> TokenStream2 {
    let name = &input.ident;

    let mut valid = vec![];

    match input.data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => {
                for fnamed in &fields.named {
                    let fname = match &fnamed.ident {
                        Some(fname) => fname,
                        None => unimplemented!("struct fields must be named")
                    };

                    let mut vld: Option<String> = None;
                    for attr in &fnamed.attrs {
                        match attr.parse_meta() {
                            Ok(syn::Meta::NameValue(attr)) => {
                                if attr.path.is_ident("valid") {
                                    match attr.lit {
                                        syn::Lit::Str(lit) => {
                                            vld = Some(lit.value());
                                        },
                                        _ => ()
                                    }
                                }
                            },
                            _ => ()
                        };
                    }

                    let vld = match vld {
                        Some(vld) => match &*vld {
                            "all" => quote! { is_all },
                            "auth" => quote! { is_auth },
                            "self" => quote! { is_self },
                            _ => unimplemented!("valid attr must be set to one of all/auth/self")
                        },
                        _ => unimplemented!("valid attr must be set")
                    };

                    let fnameq = format!("{}::{}", name, fname);
                    valid.push(quote! {
                        #vld(#fnameq, &self.#fname)?;
                    });
                }
            },
            _ => unimplemented!("struct fields must be named")

        },
        _ => unimplemented!("must be derived on a struct"),
    };

    quote! {
        fn is_valid(&self) -> Result<bool, String> {
            #(#valid)*

            Ok(true)
        }
    }
}

fn default_fn(input: &DeriveInput) -> TokenStream2 {
    let name = &input.ident;

    let mut default = vec![];

    match input.data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => {
                for fnamed in &fields.named {
                    let fname = match &fnamed.ident {
                        Some(fname) => fname,
                        None => unimplemented!("struct fields must be named")
                    };

                    let mut def: Option<String> = None;
                    for attr in &fnamed.attrs {
                        match attr.parse_meta() {
                            Ok(syn::Meta::NameValue(attr)) => {
                                if attr.path.is_ident("default") {
                                    match attr.lit {
                                        syn::Lit::Str(lit) => {
                                            def = Some(lit.value());
                                        },
                                        _ => ()
                                    }
                                }
                            },
                            _ => ()
                        };
                    }

                    // a default = "x" value is required
                    if
                        def.is_none()
                        || (
                            &*def.as_ref().unwrap() != "admin"
                            && &*def.as_ref().unwrap() != "disabled"
                            && &*def.as_ref().unwrap() != "public"
                            && &*def.as_ref().unwrap() != "self"
                            && &*def.as_ref().unwrap() != "user"

                        )
                    {
                        unimplemented!("default attr must be set to one of admin/disabled/public/self/user");
                    }

                    let def = def.unwrap();
                    default.push(quote! {
                        #fname: String::from(#def),
                    });
                }
            },
            _ => unimplemented!("struct fields must be named")

        },
        _ => unimplemented!("must be derived on a struct"),
    };

    quote! {
        fn default() -> Self {
            #name  {
                #(#default)*
            }
        }
    }
}

fn parse_fn(input: &DeriveInput) -> TokenStream2 {
    let name = &input.ident;
    let nameq = format!("{}", name);

    let mut some = vec![];
    let mut none = vec![];

    match input.data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => {
                for fnamed in &fields.named {
                    let fname = match &fnamed.ident {
                        Some(fname) => fname,
                        None => unimplemented!("struct fields must be named")
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
            _ => unimplemented!("struct fields must be named")

        },
        _ => unimplemented!("must be derived on a struct"),
    };

    quote! {
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
}
