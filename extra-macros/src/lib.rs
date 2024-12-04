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
            let mut vec_link_fields_str = Vec::default();
            let mut vec_link_types: Vec<proc_macro2::TokenStream> = Vec::default();
            let mut var_fields: Vec<proc_macro2::TokenStream> = Vec::default();
            let mut data_fields = Vec::default();
            let mut data_types = Vec::default();
            fn inner_type(type_path: &TypePath) -> proc_macro2::TokenStream {
                match &type_path.path.segments.first().unwrap().arguments {
                    PathArguments::AngleBracketed(arg) => {
                        arg.args.first().unwrap().to_token_stream()
                    }
                    _ => unimplemented!(),
                }
            }
            for field in fields.iter_mut() {
                let ty = &field.ty;
                let field_ident = field.ident.clone().unwrap();
                match ty {
                    syn::Type::Path(type_path) => {
                        let type_ident = &type_path.path.segments.first().unwrap().ident;
                        if type_ident == "Vec" {
                            vec_link_fields_str.push(field_ident.to_string());
                            vec_link_fields.push(field_ident);
                            vec_link_types.push(inner_type(type_path).into());
                        } else if type_ident == "Option" {
                            unit_link_fields_str.push(field_ident.to_string());
                            unit_link_fields.push(field_ident);
                            unit_link_types.push(inner_type(type_path).into());
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
            let data_from_dir = match nt {
                NodeType::Name => quote! {
                    let data = &format!("\"{}\"", dir.path().file_name()?.to_str()?);
                },
                NodeType::Data => quote! {
                    let data = dir.get_file(dir.path().join("data.ron"))?.contents_utf8()?;
                },
                NodeType::OnlyData => quote! {
                    let data = dir.get_file(format!("{path}.ron"))?.contents_utf8()?;
                },
            }
            .into_token_stream();
            let inner_data_from_dir = match nt {
                NodeType::Name |
                NodeType::Data => quote! {
                    #(s.#unit_link_fields = #unit_link_types::from_dir(format!("{path}/{}", #unit_link_fields_str), dir);)*
                    #(s.#vec_link_fields = dir
                        .get_dir(format!("{path}/{}", #vec_link_fields_str))
                        .into_iter()
                        .flat_map(|d| d.dirs())
                        .filter_map(|d| #vec_link_types::from_dir(d.path().to_string_lossy().to_string(), d))
                        .collect_vec();)*
                },
                NodeType::OnlyData => quote! {},
            }.into_token_stream();
            let data_type_ident = quote! { (#(#data_types),*) };
            if let Fields::Named(ref mut fields) = fields {
                fields.named.push(
                    Field::parse_named
                        .parse2(quote! { pub entity: Option<Entity> })
                        .unwrap(),
                );
            }
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
                            Err(e) => panic!("{} parsing error from {data}: {e}", self.kind()),
                        }
                    }
                    fn from_dir(path: String, dir: &Dir) -> Option<Self> {
                        dbg!(dir);
                        #data_from_dir
                        let mut s = Self::from_data(data);
                        #inner_data_from_dir
                        Some(s)
                    }
                    fn unpack(mut self, entity: Entity, commands: &mut Commands) {
                        debug!("Unpack {} into {entity}", self.kind());
                        #(
                            if let Some(d) = self.#unit_link_fields.take() {
                                d.unpack(entity, commands);
                            }
                        )*
                        #(
                            for d in std::mem::take(&mut self.#vec_link_fields) {
                                let entity = commands.spawn_empty().set_parent(entity).id();
                                d.unpack(entity, commands);
                            }
                        )*
                        commands.entity(entity).insert(self);
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
