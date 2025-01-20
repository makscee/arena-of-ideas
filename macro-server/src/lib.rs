use proc_macro::TokenStream;
use schema::*;
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
            let ParsedNodeFields {
                option_link_fields,
                option_link_fields_str,
                option_link_types,
                vec_link_fields,
                vec_link_fields_str,
                vec_link_types,
                vec_box_link_fields,
                vec_box_link_fields_str,
                vec_box_link_types,
                var_fields: _,
                var_types: _,
                data_fields: _,
                data_types: _,
                data_type_ident,
                all_data_fields,
                all_data_types: _,
            } = parse_node_fields(fields);
            let strings_conversions = strings_conversions(
                &option_link_fields,
                &option_link_fields_str,
                &option_link_types,
                &vec_link_fields,
                &vec_link_fields_str,
                &vec_link_types,
                &vec_box_link_fields,
                &vec_box_link_fields_str,
                &vec_box_link_types,
            );
            let table_conversions = table_conversions(
                &option_link_fields,
                &option_link_types,
                &vec_link_fields,
                &vec_link_types,
                &vec_box_link_fields,
                &vec_box_link_types,
            );
            quote! {
                #[derive(Default)]
                #input
                impl Node for #struct_ident {
                    #strings_conversions
                    #table_conversions
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
    quote! {
        #[derive(spacetimedb::SpacetimeType)]
        #input
    }
    .into()
}
