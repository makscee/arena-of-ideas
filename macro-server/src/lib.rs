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
                component_fields,
                component_fields_str,
                component_types,
                child_fields,
                child_fields_str,
                child_types,
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
                &component_fields,
                &component_fields_str,
                &component_types,
                &child_fields,
                &child_fields_str,
                &child_types,
            );
            let table_conversions = table_conversions(
                &component_fields,
                &component_types,
                &child_fields,
                &child_types,
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
                &component_fields,
                &component_types,
            );
            let common_trait = common_node_trait_fns(struct_ident, &component_types, &child_types);
            let component_link_fields_load = component_fields
                .iter()
                .map(|i| Ident::new(&format!("{i}_load"), Span::call_site()))
                .collect_vec();
            let child_link_fields_load = child_fields
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
                            #component_fields: #component_types,
                        )*
                        #(
                            #child_fields: Vec<#child_types>,
                        )*
                    ) -> Self {
                        let d = Self {
                            id: ctx.next_id(),
                            parent,
                            #(
                                #all_data_fields,
                            )*
                            #(
                                #component_fields: Some(#component_fields),
                            )*
                            #(
                                #child_fields,
                            )*
                        };
                        d.insert_self(ctx);
                        d
                    }
                    #(
                        pub fn #component_link_fields_load<'a>(&'a mut self, ctx: &ReducerContext) -> Result<&'a mut #component_types, String> {
                            let id = self.id();
                            if self.#component_fields.is_none() {
                                self.#component_fields = Some(self.find_child::<#component_types>(ctx)?);
                            }
                            self.#component_fields
                                .as_mut()
                                .to_e_s_fn(|| format!("{} not found for {}", #component_types::kind_s(), id))
                        }
                    )*
                    #(
                        pub fn #child_link_fields_load<'a>(&'a mut self, ctx: &ReducerContext) -> Result<&'a mut Vec<#child_types>, String> {
                            if self.#child_fields.is_empty() {
                                self.#child_fields = #child_types::collect_children_of_id(ctx, self.id());
                            }
                            if self.#child_fields.is_empty() {
                                return Err(format!("No {} children found for {}", #child_types::kind_s(), self.id()));
                            }
                            Ok(&mut self.#child_fields)
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
                    #common_trait
                    fn id(&self) -> u64 {
                        self.id
                    }
                    fn get_id(&self) -> Option<u64> {
                        Some(self.id)
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
                            if let Some(n) = self.#component_fields.as_ref() {
                                d.#component_fields = Some(n.clone(ctx, d.id));
                            }
                        )*
                        #(
                            for n in &self.#child_fields {
                                d.#child_fields.push(n.clone(ctx, d.id));
                            }
                        )*
                        d
                    }
                    fn fill_from_incubator(mut self, ctx: &ReducerContext) -> Self {
                        #(
                            self.#component_fields = self.find_incubator_component(ctx);
                            self.#component_fields = self.#component_fields.map(|n| n.fill_from_incubator(ctx));
                        )*
                        #(
                            self.#child_fields = self.collect_incubator_children(ctx);
                            self.#child_fields = self
                                .#child_fields
                                .into_iter()
                                .map(|n| n.fill_from_incubator(ctx))
                                .collect();
                        )*
                        self
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
                    pub fn parse_and_reassign_ids(nodes: &Vec<TNode>, next_id: &mut u64) -> Result<Vec<TNode>, ExpressionError> {
                        let root = &nodes[0];
                        let kind = root.kind.to_kind();
                        match kind {
                            Self::None => Err("Can't convert None kind".into()),
                            #(#struct_ident::#variants => {
                                let mut d = #variants::from_tnodes(root.id, nodes).to_e_fn(|| format!("Failed to parse"))?;
                                d.reassign_ids(next_id);
                                Ok(d.to_tnodes())
                            })*
                        }
                    }
                }
            }
            .into()
        }
        _ => unimplemented!(),
    }
}
