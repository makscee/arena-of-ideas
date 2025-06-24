use itertools::Itertools;
use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::quote;
use schema::*;
use std::env;
use std::fs;
use std::path::Path;
use syn::parse::Parser;
use syn::*;

fn main() {
    println!("cargo:rerun-if-changed=../nodes/src/raw_nodes.rs");
    println!("cargo::rerun-if-changed=build.rs");

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("server_impls.rs");

    // Read the raw nodes file from the nodes crate
    let input =
        fs::read_to_string("../raw-nodes/src/raw_nodes.rs").expect("Failed to read raw_nodes.rs");
    let syntax_tree = parse_file(&input).expect("Failed to parse raw_nodes.rs");

    let mut structs = Vec::new();
    let mut names: Vec<_> = Vec::new();
    for item in syntax_tree.items {
        if let Item::Struct(item_struct) = item {
            names.push(item_struct.ident.clone());
            structs.push(item_struct);
        }
    }
    let node_kind_impls = generate_node_kinds(names);
    let server_impls: Vec<_> = structs
        .into_iter()
        .map(|item| generate_impl(item))
        .collect();
    let output = quote! {
        #(#server_impls)*
        #node_kind_impls
    };

    // Parse the generated code and format it
    let formatted_code = match syn::parse_file(&output.to_string()) {
        Ok(parsed) => prettyplease::unparse(&parsed),
        Err(_) => {
            // If parsing fails, fall back to unformatted output
            eprintln!(
                "Warning: Failed to parse generated code for formatting, using unformatted output"
            );
            output.to_string()
        }
    };

    fs::write(&dest_path, formatted_code).expect("Failed to write server implementations file");
}

fn generate_node_kinds(names: Vec<Ident>) -> TokenStream {
    quote! {
        #[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, Display, EnumIter, PartialEq, Eq, PartialOrd, Ord, strum_macros::EnumString, strum_macros::AsRefStr, Hash)]
        pub enum NodeKind {
            #[default]
            None,
            #(
                #names,
            )*
        }
        pub trait NodeKindExt {
            fn to_kind(&self) -> NodeKind;
        }

        impl NodeKindExt for String {
            fn to_kind(&self) -> NodeKind {
                NodeKind::from_str(self).unwrap()
            }
        }
        impl NodeKind {
            pub fn component_kinds(self) -> HashSet<Self> {
                match self {
                    NodeKind::None => default(),
                    #(
                        Self::#names => {
                            #names::component_kinds()
                        }
                    )*
                }
            }
            pub fn children_kinds(self) -> HashSet<Self> {
                match self {
                    NodeKind::None => default(),
                    #(
                        Self::#names => {
                            #names::children_kinds()
                        }
                    )*
                }
            }
            pub fn convert(self, data: &str) -> Result<TNode, ExpressionError> {
                match self {
                    Self::None => Err("Can't convert None kind".into()),
                    #(Self::#names => {
                        let mut d = #names::default();
                        d.inject_data(data)?;
                        Ok(d.to_tnode())
                    }
                    )*
                }
            }
            pub fn delete_with_components(self, ctx: &ReducerContext, id: u64) -> Result<(), String> {
                match self {
                    Self::None => unreachable!(),
                    #(
                        Self::#names => {
                            #names::get(ctx, id).to_custom_e_s_fn(|| format!("Failed to get {self}#{id}"))?.delete_with_components(ctx);
                        }
                    )*
                }
                Ok(())
            }
        }
    }
}

fn generate_impl(mut item: ItemStruct) -> TokenStream {
    let struct_ident = &item.ident;
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
    } = parse_node_fields(&item.fields);

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
    if let Fields::Named(fields) = &mut item.fields {
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
    let parent_link_add = parent_fields
        .iter()
        .map(|i| Ident::new(&format!("{i}_add"), Span::call_site()))
        .collect_vec();
    let parent_link_remove = parent_fields
        .iter()
        .map(|i| Ident::new(&format!("{i}_remove"), Span::call_site()))
        .collect_vec();

    quote! {
        #[derive(Default, Debug)]
        pub #item
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
                    ..Default::default()
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
                    self.id.add_parent(ctx, #one_fields.id)?;
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
                        self.#one_fields = self.parent::<#one_types>(ctx);
                    }
                    self.#one_fields
                        .as_mut()
                        .to_custom_e_s_fn(|| format!("{} not found for {}", #one_types::kind_s(), id))
                }
            )*
            #(
                pub fn #many_link_fields_load<'a>(&'a mut self, ctx: &ReducerContext) -> Result<&'a mut Vec<#many_types>, String> {
                    if self.#many_fields.is_empty() {
                        self.#many_fields = self.collect_children::<#many_types>(ctx);
                    }
                    if self.#many_fields.is_empty() {
                        return Err(format!("No {} children found for {}", #many_types::kind_s(), self.id()));
                    }
                    Ok(&mut self.#many_fields)
                }
            )*
            #(
                pub fn #parent_link_add(&mut self, ctx: &ReducerContext, id: u64) -> Result<(), String> {
                    if self.#parent_fields.ids.contains(&id) {
                        return Err(format!(
                            "{}#{} already has parent#{id}",
                            self.kind(),
                            self.id
                        ));
                    }
                    self.#parent_fields.ids.push(id);
                    self.id.add_parent(ctx, id)?;
                    self.update_self(ctx);
                    Ok(())
                }
            )*
            #(
                pub fn #parent_link_remove(&mut self, ctx: &ReducerContext, id: u64) -> Result<(), String> {
                    let Some(i) = self.#parent_fields.ids.iter().position(|u| *u == id) else {
                        return Err(format!(
                            "{}#{} does not have parent#{id}",
                            self.kind(),
                            self.id
                        ));
                    };
                    self.#parent_fields.ids.remove(i);
                    self.id.remove_parent(ctx, id)?;
                    self.update_self(ctx);
                    Ok(())
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
            fn set_owner(&mut self, id: u64) {
                self.owner = id;
            }
            fn clone_self(&self, ctx: &ReducerContext, owner: u64) -> Self {
                let mut d = Self::new(
                    ctx,
                    owner,
                    #(
                        self.#all_data_fields.clone(),
                    )*
                );
                d
            }
            fn clone(&self, ctx: &ReducerContext, owner: u64, remap: &mut HashMap<u64, u64>) -> Self {
                let mut d = self.clone_self(ctx, owner);
                remap.insert(self.id, d.id);
                #(
                    if let Some(n) = self.#one_fields.as_ref() {
                        let n = n.clone(ctx, owner, remap);
                        d.id.add_parent(ctx, n.id).unwrap();
                        d.#one_fields = Some(n);
                    }
                )*
                #(
                    for n in &self.#many_fields {
                        let n = n.clone(ctx, owner, remap);
                        d.id.add_child(ctx, n.id).unwrap();
                        d.#many_fields.push(n);
                    }
                )*
                d
            }
            fn collect_ids(&self) -> Vec<u64> {
                let mut v = [self.id].to_vec();
                #(
                    if let Some(n) = self.#one_fields.as_ref() {
                        v.extend(n.collect_ids());
                    }
                )*
                #(
                    for n in &self.#many_fields {
                        v.extend(n.collect_ids());
                    }
                )*
                v
            }
            fn solidify_links(&self, ctx: &ReducerContext) -> Result<(), String> {
                #(
                    if let Some(n) = &self.#one_fields {
                        TNodeLink::solidify(ctx, n.id, self.id)?;
                        n.solidify_links(ctx);
                    }
                )*
                #(
                    for n in &self.#many_fields {
                        TNodeLink::solidify(ctx, self.id, n.id)?;
                        n.solidify_links(ctx);
                    }
                )*
                Ok(())
            }
            fn with_components(&mut self, ctx: &ReducerContext) -> &mut Self {
                #(
                    self.#one_fields = self.parent::<#one_types>(ctx)
                        .map(|mut d| std::mem::take(d.with_components(ctx)
                            .with_children(ctx))
                        );
                )*
                self
            }
            fn with_children(&mut self, ctx: &ReducerContext) -> &mut Self {
                #(
                    self.#many_fields = self.collect_children::<#many_types>(ctx)
                        .into_iter()
                        .map(|mut n| std::mem::take(n.with_components(ctx).with_children(ctx)))
                        .collect();
                )*
                self
            }
            fn save(&self, ctx: &ReducerContext) {
                self.update_self(ctx);
                #(
                    if let Some(d) = &self.#one_fields {
                        d.save(ctx);
                    }
                )*
                #(
                    for d in &self.#many_fields {
                        d.save(ctx);
                    }
                )*
            }
            fn delete_with_components(&self, ctx: &ReducerContext) {
                #(
                    if let Some(n) = self.parent::<#one_types>(ctx) {
                        n.delete_with_components(ctx);
                    }
                )*
                #(
                    for n in self.collect_children::<#many_types>(ctx) {
                        n.delete_with_components(ctx);
                    }
                )*
                self.delete_self(ctx);
            }
        }
    }
    .into()
}
