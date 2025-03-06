use itertools::Itertools;
use parse::Parser;
use proc_macro::TokenStream;
use proc_macro2::Span;
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
                fields
                    .named
                    .push(Field::parse_named.parse2(quote! { pub id: u64 }).unwrap());
                fields.named.push(
                    Field::parse_named
                        .parse2(quote! { pub parent: u64 })
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
            let component_link_fields_load = component_link_fields
                .iter()
                .map(|i| Ident::new(&format!("{i}_load"), Span::call_site()))
                .collect_vec();
            let child_link_fields_load = child_link_fields
                .iter()
                .map(|i| Ident::new(&format!("{i}_load"), Span::call_site()))
                .collect_vec();
            quote! {
                #[derive(Default, Debug)]
                #input
                #common
                impl #struct_ident {
                    pub fn new(
                        ctx: &ReducerContext,
                        parent: u64,
                        #(
                            #all_data_fields: #all_data_types,
                        )*
                    ) -> Self {
                        let d = Self {
                            id: ctx.next_id(),
                            parent,
                            #(
                                #all_data_fields,
                            )*
                            ..default()
                        };
                        d.insert_self(ctx);
                        d
                    }
                    pub fn new_full(
                        ctx: &ReducerContext,
                        parent: u64,
                        #(
                            #all_data_fields: #all_data_types,
                        )*
                        #(
                            #component_link_fields: #component_link_types,
                        )*
                        #(
                            #child_link_fields: Vec<#child_link_types>,
                        )*
                    ) -> Self {
                        let d = Self {
                            id: ctx.next_id(),
                            parent,
                            #(
                                #all_data_fields,
                            )*
                            #(
                                #component_link_fields: Some(#component_link_fields),
                            )*
                            #(
                                #child_link_fields,
                            )*
                        };
                        d.insert_self(ctx);
                        d
                    }
                    #(
                        pub fn #component_link_fields_load<'a>(&'a mut self, ctx: &ReducerContext) -> Result<&'a mut #component_link_types, String> {
                            let id = self.id();
                            if self.#component_link_fields.is_none() {
                                self.#component_link_fields = Some(self.find_child::<#component_link_types>(ctx)?);
                            }
                            self.#component_link_fields
                                .as_mut()
                                .to_e_s_fn(|| format!("{} not found for {}", #component_link_types::kind_s(), id))
                        }
                    )*
                    #(
                        pub fn #child_link_fields_load<'a>(&'a mut self, ctx: &ReducerContext) -> Result<&'a mut Vec<#child_link_types>, String> {
                            if self.#child_link_fields.is_empty() {
                                self.#child_link_fields = #child_link_types::collect_children_of_id(ctx, self.id());
                            }
                            if self.#child_link_fields.is_empty() {
                                return Err(format!("No {} children found for {}", #child_link_types::kind_s(), self.id()));
                            }
                            Ok(&mut self.#child_link_fields)
                        }
                    )*
                    pub fn find_by_data(
                        ctx: &ReducerContext,
                        #(
                            #all_data_fields: #all_data_types,
                        )*
                    ) -> Option<Self> {
                        let kind = Self::kind_s().to_string();
                        let data = Self {
                            #(
                                #all_data_fields,
                            )*
                            ..default()
                        }.get_data();
                        let n = ctx
                            .db
                            .nodes_world()
                            .data()
                            .filter(&data)
                            .find(|n| n.kind == kind);
                        n.map(|n| n.to_node().unwrap())
                    }
                }
                impl StringData for #struct_ident {
                    fn get_data(&self) -> String {
                        ron::to_string(&(#(&self.#all_data_fields),*)).unwrap()
                    }
                    fn inject_data(&mut self, data: &str) -> Result<(), ExpressionError> {
                        match ron::from_str::<#data_type_ident>(data) {
                            Ok(v) => {(#(self.#all_data_fields),*) = v; Ok(())}
                            Err(e) => Err(format!("{} parsing error from {data}: {e}", self.kind()).into()),
                        }
                    }
                }
                impl Node for #struct_ident {
                    #strings_conversions
                    #table_conversions
                    fn id(&self) -> u64 {
                        self.id
                    }
                    fn set_id(&mut self, id: u64) {
                        self.id = id;
                    }
                    fn parent(&self) -> u64 {
                        self.parent
                    }
                    fn set_parent(&mut self, id: u64) {
                        self.parent = id;
                    }
                    fn clone(&self, ctx: &ReducerContext, parent: u64) -> Self {
                        let mut d = Self::new(
                            ctx, parent,
                            #(
                                self.#all_data_fields.clone(),
                            )*
                        );
                        d.parent = parent;
                        #(
                            if let Some(n) = self.#component_link_fields.as_ref() {
                                d.#component_link_fields = Some(n.clone(ctx, d.id));
                            }
                        )*
                        #(
                            for n in &self.#child_link_fields {
                                d.#child_link_fields.push(n.clone(ctx, d.id));
                            }
                        )*
                        d
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
    let mut input = parse_macro_input!(item as syn::DeriveInput);
    let struct_ident = &input.ident;
    match &mut input.data {
        Data::Enum(DataEnum {
            enum_token: _,
            brace_token: _,
            variants,
        }) => {
            let variants = variants
                .iter()
                .map(|v| v.ident.clone())
                .filter(|v| v != "None")
                .collect_vec();
            quote! {
                #[derive(spacetimedb::SpacetimeType)]
                #input
                impl NodeKind {
                    pub fn convert(self, data: &str) -> Result<TNode, ExpressionError> {
                        match self {
                            Self::None => Err("Can't convert None kind".into()),
                            #(#struct_ident::#variants => {
                                let mut d = #variants::default();
                                d.inject_data(data)?;
                                Ok(d.to_tnode())
                            }
                            )*
                        }
                    }
                    pub fn save_from_strings(self, ctx: &ReducerContext, parent: u64, datas: &Vec<String>) -> Result<(), ExpressionError> {
                        match self {
                            Self::None => {return Err("Can't convert None kind".into());}
                            #(#struct_ident::#variants => {
                                let mut d = #variants::from_strings(0, datas).to_e_fn(|| format!("Failed to convert {self} from {datas:?}"))?;
                                d.clone(ctx, parent);
                            }
                            )*
                        }
                        Ok(())
                    }
                }
            }
            .into()
        }
        _ => unimplemented!(),
    }
}
