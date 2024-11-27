use itertools::Itertools;
use proc_macro::TokenStream;
use syn::{spanned::Spanned, Ident};
#[macro_use]
extern crate quote;

#[proc_macro_derive(ContentNode)]
pub fn derive_content_node(item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::DeriveInput);
    let struct_identifier = &input.ident;
    match input.data {
        syn::Data::Struct(syn::DataStruct {
            struct_token: _,
            fields,
            semi_token: _,
        }) => {
            let mut unit_fields = Vec::default();
            let mut vec_fields = Vec::default();
            let mut data: Option<Ident> = None;
            for field in fields.into_iter() {
                let ty = field.ty;
                let ident = field.ident.unwrap();
                match ty {
                    syn::Type::Path(type_path) => {
                        let field_ident = &type_path.path.segments.first().unwrap().ident;
                        if field_ident == "String" {
                            if data.is_none() {
                                data = Some(ident);
                            } else {
                                panic!("There should be a single String data field");
                            }
                        } else if field_ident == "Vec" {
                            vec_fields.push(ident);
                        } else {
                            unit_fields.push(ident);
                        }
                    }
                    _ => unimplemented!(),
                }
            }
            let data = data.expect("String data field is missing");
            quote! {
                impl ContentNode for #struct_identifier {
                    fn kind(&self) -> ContentKind {
                        ContentKind::#struct_identifier
                    }
                    fn data(&self) -> &String {
                        &self.#data
                    }
                    fn data_mut(&mut self) -> &mut String {
                        &mut self.#data
                    }
                    fn links(&self, f: fn(&dyn ContentNode)) {
                        #(
                            f(&self.#unit_fields);
                        )*
                        #(
                            for d in &self.#vec_fields {
                                f(d)
                            }
                        )*
                    }
                    fn walk(&self, f: fn(&dyn ContentNode)) {
                        f(self);
                        #(
                            &self.#unit_fields.walk(f);
                        )*
                        #(
                            for d in &self.#vec_fields {
                                d.walk(f);
                            }
                        )*
                    }
                }
            }
            .into()
        }
        _ => unimplemented!(),
    }
}
