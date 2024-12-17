use proc_macro::TokenStream;
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
                option_link_fields: _,
                option_link_fields_str: _,
                option_link_types: _,
                vec_link_fields: _,
                vec_link_fields_str: _,
                vec_link_types: _,
                vec_box_link_fields: _,
                vec_box_link_fields_str: _,
                vec_box_link_types: _,
                var_fields: _,
                var_types: _,
                data_fields: _,
                data_types: _,
                data_type_ident,
                all_data_fields,
                all_data_types: _,
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
#[proc_macro_attribute]
pub fn node_kinds(_: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as syn::DeriveInput);
    quote! {#input}.into()
}
