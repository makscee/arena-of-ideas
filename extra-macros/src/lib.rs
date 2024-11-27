use std::collections::HashMap;

use proc_macro::TokenStream;
use syn::Ident;
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
            let mut vars: Vec<proc_macro2::TokenStream> = Vec::default();
            for field in fields.into_iter() {
                let ty = field.ty;
                let field_ident = field.ident.unwrap();
                match ty {
                    syn::Type::Path(type_path) => {
                        let type_ident = &type_path.path.segments.first().unwrap().ident;
                        if type_ident == "Vec" {
                            vec_fields.push(field_ident);
                        } else if type_ident == "i32" || type_ident == "String" {
                            vars.push(quote! {VarName::#field_ident => return Some(VarValue::#type_ident(self.#field_ident.clone()))}.into());
                        } else {
                            unit_fields.push(field_ident);
                        }
                    }
                    _ => unimplemented!(),
                }
            }
            quote! {
                impl ContentNode for #struct_identifier {
                    fn kind(&self) -> ContentKind {
                        ContentKind::#struct_identifier
                    }
                    fn get_var(&self, var: VarName) -> Option<VarValue> {
                        match var {
                            #(
                                #vars,
                            )*
                            _ => {
                                #(
                                    if let Some(v) = &self.#unit_fields.get_var(var) {
                                        return Some(v.clone());
                                    }
                                )*
                            }
                        };
                        None
                    }
                    fn walk(&self, f: fn(&dyn ContentNode)) {
                        f(self);
                        #(
                            self.#unit_fields.walk(f);
                        )*
                        #(
                            for d in self.#vec_fields.iter() {
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
