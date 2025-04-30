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
                one_fields,
                one_fields_str,
                one_types,
                many_fields,
                many_fields_str,
                many_types,
                var_fields: _,
                var_types: _,
                data_fields: _,
                data_fields_str: _,
                data_types: _,
                data_type_ident,
                all_data_fields,
                all_data_types,
                parent_fields,
                parent_types,
            } = parse_node_fields(fields);
            let strings_conversions = strings_conversions(
                &one_fields,
                &one_fields_str,
                &one_types,
                &many_fields,
                &many_fields_str,
                &many_types,
                &parent_fields,
                &parent_types,
            );
            let table_conversions =
                table_conversions(&one_fields, &one_types, &many_fields, &many_types);
            if let Fields::Named(ref mut fields) = fields {
                fields
                    .named
                    .push(Field::parse_named.parse2(quote! { pub id: u64 }).unwrap());
                fields.named.push(
                    Field::parse_named
                        .parse2(quote! { pub owner: u64 })
                        .unwrap(),
                );
            }
            let common = common_node_fns(
                struct_ident,
                &all_data_fields,
                &all_data_types,
                &one_fields,
                &one_types,
            );
            let common_trait = common_node_trait_fns(struct_ident, &one_types, &many_types);
            let one_link_fields_load = one_fields
                .iter()
                .map(|i| Ident::new(&format!("{i}_load"), Span::call_site()))
                .collect_vec();
            let many_link_fields_load = many_fields
                .iter()
                .map(|i| Ident::new(&format!("{i}_load"), Span::call_site()))
                .collect_vec();
            let one_link_fields_set = one_fields
                .iter()
                .map(|i| Ident::new(&format!("{i}_set"), Span::call_site()))
                .collect_vec();
            let many_link_fields_set = many_fields
                .iter()
                .map(|i| Ident::new(&format!("{i}_add"), Span::call_site()))
                .collect_vec();
            quote! {
                #[derive(Default, Debug)]
                #input
                #common
                impl #struct_ident {
                    pub fn new(
                        ctx: &ReducerContext,
                        owner: u64,
                        #(
                            #all_data_fields: #all_data_types,
                        )*
                    ) -> Self {
                        let d = Self {
                            id: ctx.next_id(),
                            owner,
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
                        owner: u64,
                        #(
                            #all_data_fields: #all_data_types,
                        )*
                        #(
                            #one_fields: #one_types,
                        )*
                        #(
                            #many_fields: Vec<#many_types>,
                        )*
                    ) -> Self {
                        let d = Self {
                            id: ctx.next_id(),
                            owner,
                            #(
                                #all_data_fields,
                            )*
                            #(
                                #one_fields: Some(#one_fields),
                            )*
                            #(
                                #many_fields,
                            )*
                            ..default()
                        };
                        d.insert_self(ctx);
                        d
                    }
                    #(
                        pub fn #one_link_fields_set(&mut self, ctx: &ReducerContext, mut #one_fields: #one_types) -> Result<&mut Self, String> {
                            self.id.add_child(ctx, #one_fields.id)?;
                            self.#one_fields = Some(#one_fields);
                            Ok(self)
                        }
                    )*
                    #(
                        pub fn #many_link_fields_set(&mut self, ctx: &ReducerContext, mut #many_fields: #many_types) -> Result<&mut Self, String> {
                            self.id.add_child(ctx, #many_fields.id)?;
                            self.#many_fields.push(#many_fields);
                            Ok(self)
                        }
                    )*
                    #(
                        pub fn #one_link_fields_load<'a>(&'a mut self, ctx: &ReducerContext) -> Result<&'a mut #one_types, String> {
                            let id = self.id();
                            if self.#one_fields.is_none() {
                                self.#one_fields = Some(self.find_child::<#one_types>(ctx)?);
                            }
                            self.#one_fields
                                .as_mut()
                                .to_custom_e_s_fn(|| format!("{} not found for {}", #one_types::kind_s(), id))
                        }
                    )*
                    #(
                        pub fn #many_link_fields_load<'a>(&'a mut self, ctx: &ReducerContext) -> Result<&'a mut Vec<#many_types>, String> {
                            if self.#many_fields.is_empty() {
                                self.#many_fields = #many_types::collect_children_of_id(ctx, self.id());
                            }
                            if self.#many_fields.is_empty() {
                                return Err(format!("No {} children found for {}", #many_types::kind_s(), self.id()));
                            }
                            Ok(&mut self.#many_fields)
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
                impl Node for #struct_ident {
                    #strings_conversions
                    #table_conversions
                    #common_trait
                    fn id(&self) -> u64 {
                        self.id
                    }
                    fn set_id(&mut self, id: u64) {
                        self.id = id;
                    }
                    fn owner(&self) -> u64 {
                        self.owner
                    }
                    fn clone_self(&self, ctx: &ReducerContext) -> Self {
                        let mut d = Self::new(
                            ctx,
                            self.owner,
                            #(
                                self.#all_data_fields.clone(),
                            )*
                        );
                        d
                    }
                    fn clone(&self, ctx: &ReducerContext, remap: &mut HashMap<u64, u64>) -> Self {
                        let mut d = self.clone_self(ctx);
                        remap.insert(self.id, d.id);
                        #(
                            if let Some(n) = self.#one_fields.as_ref() {
                                d.#one_fields = Some(n.clone(ctx, remap));
                            }
                        )*
                        #(
                            for n in &self.#many_fields {
                                d.#many_fields.push(n.clone(ctx, remap));
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
                }
            }
            .into()
        }
        _ => unimplemented!(),
    }
}
