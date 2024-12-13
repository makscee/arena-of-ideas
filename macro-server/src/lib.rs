use darling::FromMeta;
use itertools::Itertools;
use parse::Parser;
use proc_macro::TokenStream;
use punctuated::Punctuated;
use quote::ToTokens;
use schema::macro_fn::parse_node_fields;
use syn::*;
#[macro_use]
extern crate quote;

#[proc_macro_attribute]
pub fn node(_: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as syn::DeriveInput);
    let struct_ident = &input.ident;
    match &mut input.data {
        syn::Data::Struct(syn::DataStruct {
            struct_token: _,
            fields,
            semi_token: _,
        }) => {
            let schema::macro_fn::ParsedNodeFields {
                option_link_fields,
                option_link_fields_str,
                option_link_types,
                vec_link_fields,
                vec_link_fields_str,
                vec_link_types,
                vec_box_link_fields,
                vec_box_link_fields_str,
                vec_box_link_types,
                var_fields,
                var_types,
                data_fields,
                data_types,
                data_type_ident,
                all_data_fields,
                all_data_types,
            } = parse_node_fields(fields);
            quote! {
                #[derive(Default)]
                #input
                impl Node for #struct_ident {
                    fn get_data(&self) -> String {
                        ron::to_string(&(#(&self.#all_data_fields),*)).unwrap()
                    }
                    fn inject_data(&mut self, data: &str) {
                        match ron::from_str::<#data_type_ident>(data) {
                            Ok(v) => (#(self.#all_data_fields),*) = v,
                            Err(e) => panic!("{} parsing error from {data}: {e}", self.kind()),
                        }
                    }
                }
            }
            .into()
        }
        _ => unimplemented!(),
    }
}
