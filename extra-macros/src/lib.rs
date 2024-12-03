use darling::FromMeta;
use parse::Parser;
use proc_macro::TokenStream;
use quote::ToTokens;
use syn::*;
#[macro_use]
extern crate quote;

#[proc_macro_attribute]
pub fn content_node(args: TokenStream, item: TokenStream) -> TokenStream {
    let _ = parse_macro_input!(args as parse::Nothing);
    let mut input = parse_macro_input!(item as syn::DeriveInput);
    let struct_ident = &input.ident;

    enum NodeType {
        Name,
        Data,
        OnlyData,
    }

    match &mut input.data {
        syn::Data::Struct(syn::DataStruct {
            struct_token: _,
            fields,
            semi_token: _,
        }) => {
            let mut unit_link_fields = Vec::default();
            let mut unit_link_fields_str = Vec::default();
            let mut unit_link_types: Vec<proc_macro2::TokenStream> = Vec::default();
            let mut vec_link_fields = Vec::default();
            let mut var_fields: Vec<proc_macro2::TokenStream> = Vec::default();
            let mut data_fields = Vec::default();
            let mut data_types = Vec::default();
            for field in fields.iter_mut() {
                let ty = &field.ty;
                let field_ident = field.ident.clone().unwrap();
                match ty {
                    syn::Type::Path(type_path) => {
                        let type_ident = &type_path.path.segments.first().unwrap().ident;
                        if type_ident == "Vec" {
                            vec_link_fields.push(field_ident);
                        } else if type_ident == "Option" {
                            unit_link_fields_str.push(field_ident.to_string());
                            unit_link_fields.push(field_ident);
                            let inner_type =
                                match &type_path.path.segments.first().unwrap().arguments {
                                    PathArguments::AngleBracketed(arg) => {
                                        arg.args.first().unwrap().to_token_stream()
                                    }
                                    _ => unimplemented!(),
                                };
                            unit_link_types.push(inner_type.into());
                        } else if type_ident == "i32"
                            || type_ident == "f32"
                            || type_ident == "String"
                        {
                            var_fields.push(quote! {VarName::#field_ident => return Some(VarValue::#type_ident(self.#field_ident.clone()))}.into());
                            data_fields.push(field_ident);
                            data_types.push(ty.clone());
                        } else {
                            data_fields.push(field_ident);
                            data_types.push(ty.clone());
                        }
                    }
                    _ => unimplemented!(),
                }
            }
            if data_types.is_empty() {
                panic!("No data fields found");
            }
            let nt = if data_fields.contains(&Ident::from_string("name").unwrap()) {
                NodeType::Name
            } else if !unit_link_fields.is_empty() {
                NodeType::Data
            } else {
                NodeType::OnlyData
            };
            let from_entry = match nt {
                NodeType::Name => {
                    quote! {
                        let Some(dir) = dir.and_then(|d| d.as_dir()) else {
                            return None;
                        };
                        let dir_name = dir.path().file_name().unwrap().to_string_lossy();
                        Some(Self {
                            name: dir_name.into(),
                            #(#unit_link_fields: #unit_link_types::from_entry(dir.get_entry(dir.path().join(#unit_link_fields_str))),)*
                        })
                    }
                }
                NodeType::Data => {
                    quote! {
                        let Some(dir) = dir.and_then(|d| d.as_dir()) else {
                            return None;
                        };
                        dir.get_file(dir.path().join("data.ron"))
                            .and_then(|f| f.contents_utf8())
                            .map(|c| {
                                let mut s = Self::from_data(c);
                                #(s.#unit_link_fields = #unit_link_types::from_entry(dir.get_entry(dir.path().join(&(#unit_link_fields_str.to_owned() + ".ron"))));)*;
                                s
                            })
                    }
                }
                NodeType::OnlyData => {
                    quote! {
                        dir.and_then(|d| d.as_file())
                            .and_then(|f| f.contents_utf8())
                            .map(|f| Self::from_data(f))
                    }
                }
            };
            let data_type_ident = quote! { (#(#data_types),*) };
            // if let Fields::Named(ref mut fields) = fields {
            //     fields.named.push(
            //         Field::parse_named
            //             .parse2(quote! { pub data: String })
            //             .unwrap(),
            //     );
            // }
            quote! {
                #[derive(Component, Clone, Default, Debug)]
                #input

                impl ContentNode for #struct_ident {
                    fn kind(&self) -> ContentKind {
                        ContentKind::#struct_ident
                    }
                    fn get_var(&self, var: VarName) -> Option<VarValue> {
                        match var {
                            #(
                                #var_fields,
                            )*
                            _ => {
                                #(
                                    if let Some(v) = self.#unit_link_fields.as_ref().and_then(|l| l.get_var(var)).clone() {
                                        return Some(v);
                                    }
                                )*
                            }
                        };
                        None
                    }
                    fn get_data(&self) -> String {
                        ron::to_string(&(#(&self.#data_fields),*)).unwrap()
                    }
                    fn inject_data(&mut self, data: &str) {
                        match ron::from_str::<#data_type_ident>(data) {
                            Ok(v) => (#(self.#data_fields),*) = v,
                            Err(e) => panic!("{e}"),
                        }
                    }
                    fn from_entry(dir: Option<&DirEntry>) -> Option<Self> {
                        dbg!(dir);
                        #from_entry
                    }
                }
                impl From<&str> for #struct_ident {
                    fn from(value: &str) -> Self {
                        Self::from_data(value)
                    }
                }
            }
            .into()
        }
        _ => unimplemented!(),
    }
}
