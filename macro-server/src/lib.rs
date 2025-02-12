use parse::Parser;
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
                component_link_fields,
                component_link_fields_str,
                component_link_types,
                child_link_fields,
                child_link_fields_str,
                child_link_types,
                var_fields: _,
                var_types: _,
                data_fields: _,
                data_fields_str: _,
                data_types: _,
                data_type_ident,
                all_data_fields,
                all_data_types,
            } = parse_node_fields(fields);
            let strings_conversions = strings_conversions(
                &component_link_fields,
                &component_link_fields_str,
                &component_link_types,
                &child_link_fields,
                &child_link_fields_str,
                &child_link_types,
            );
            let table_conversions = table_conversions(
                &component_link_fields,
                &component_link_types,
                &child_link_fields,
                &child_link_types,
            );
            if let Fields::Named(ref mut fields) = fields {
                fields.named.push(
                    Field::parse_named
                        .parse2(quote! { pub id: Option<u64> })
                        .unwrap(),
                );
            }
            let common = common_node_fns(
                struct_ident,
                &all_data_fields,
                &all_data_types,
                &component_link_fields,
                &component_link_types,
            );
            quote! {
                #[derive(Default, Debug, Clone)]
                #input
                #common
                impl Node for #struct_ident {
                    #strings_conversions
                    #table_conversions
                    fn id(&self) -> u64 {
                        self.id.expect("Id not set")
                    }
                    fn get_id(&self) -> Option<u64> {
                        self.id
                    }
                    fn set_id(&mut self, id: u64) {
                        self.id = Some(id);
                    }
                    fn clear_ids(&mut self) {
                        self.id = None;
                        #(
                            if let Some(d) = &mut self.#component_link_fields {
                                d.clear_ids();
                            }
                        )*
                        #(
                            for d in self.#child_link_fields.iter_mut() {
                                d.clear_ids();
                            }
                        )*
                    }
                    fn gather_ids(&self, data: &mut HashSet<u64>) {
                        data.extend(self.id.iter().copied());
                        #(
                            if let Some(d) = &self.#component_link_fields {
                                d.gather_ids(data);
                            }
                        )*
                        #(
                            for d in self.#child_link_fields.iter() {
                                d.gather_ids(data);
                            }
                        )*
                    }
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
