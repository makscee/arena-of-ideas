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
    println!("cargo:rerun-if-changed=../raw-nodes/src/raw_nodes.rs");
    println!("cargo::rerun-if-changed=build.rs");

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("server_impls.rs");

    // Read the raw nodes file from the raw-nodes crate
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
    let server_trait_impls = generate_server_trait_impls(&names);
    let server_impls: Vec<_> = structs
        .into_iter()
        .map(|item| generate_impl(item))
        .collect();
    let output = quote! {
        #(#server_impls)*
        #server_trait_impls
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

fn generate_server_trait_impls(names: &[Ident]) -> TokenStream {
    quote! {
        #[allow(unused)]
        #[allow(dead_code)]
        #[allow(unused_mut)]
        pub trait ServerNodeKind {
            fn owned_kinds(self) -> HashSet<NodeKind>;
            fn owned_parents(self) -> HashSet<NodeKind>;
            fn owned_children(self) -> HashSet<NodeKind>;
            fn linked_children(self) -> HashSet<NodeKind>;
            fn linked_parents(self) -> HashSet<NodeKind>;
            fn component_kinds(self) -> HashSet<NodeKind>;
            fn children_kinds(self) -> HashSet<NodeKind>;
            fn convert(self, data: &str) -> Result<TNode, ExpressionError>;
            fn delete_with_components(self, ctx: &ReducerContext, id: u64) -> Result<(), String>;
        }
        impl ServerNodeKind for NodeKind {
            fn owned_kinds(self) -> HashSet<Self> {
                match self {
                    NodeKind::None => default(),
                    #(
                        Self::#names => {
                            #names::owned_kinds()
                        }
                    )*
                }
            }
            fn owned_parents(self) -> HashSet<Self> {
                match self {
                    NodeKind::None => default(),
                    #(
                        Self::#names => {
                            #names::owned_parents()
                        }
                    )*
                }
            }
            fn owned_children(self) -> HashSet<Self> {
                match self {
                    NodeKind::None => default(),
                    #(
                        Self::#names => {
                            #names::owned_children()
                        }
                    )*
                }
            }
            fn linked_children(self) -> HashSet<Self> {
                match self {
                    NodeKind::None => default(),
                    #(
                        Self::#names => {
                            #names::linked_children()
                        }
                    )*
                }
            }
            fn linked_parents(self) -> HashSet<Self> {
                match self {
                    NodeKind::None => default(),
                    #(
                        Self::#names => {
                            #names::linked_parents()
                        }
                    )*
                }
            }
            fn component_kinds(self) -> HashSet<Self> {
                match self {
                    NodeKind::None => default(),
                    #(
                        Self::#names => {
                            #names::component_kinds()
                        }
                    )*
                }
            }
            fn children_kinds(self) -> HashSet<Self> {
                match self {
                    NodeKind::None => default(),
                    #(
                        Self::#names => {
                            #names::children_kinds()
                        }
                    )*
                }
            }
            fn convert(self, data: &str) -> Result<TNode, ExpressionError> {
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
            fn delete_with_components(self, ctx: &ReducerContext, id: u64) -> Result<(), String> {
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
    let pnf = parse_node_fields(&item.fields);

    let ParsedNodeFields {
        var_fields: _,
        var_types: _,
        data_fields: _,
        data_types: _,
        all_data_fields,
        all_data_types,
        owned_children_fields,
        owned_children_types,
        owned_parents_fields,
        owned_parents_types,
        owned_child_fields,
        owned_child_types,
        owned_parent_fields,
        owned_parent_types,
        linked_children_fields,
        linked_children_types,
        linked_parents_fields,
        linked_parents_types,
        linked_child_fields,
        linked_child_types,
        linked_parent_fields,
        linked_parent_types,
    } = &pnf;
    let (one_fields, one_types) = pnf.one_owned();
    let (many_fields, many_types) = pnf.many_owned();
    let strings_conversions = strings_conversions(
        owned_children_fields,
        owned_children_types,
        owned_parents_fields,
        owned_parents_types,
        owned_child_fields,
        owned_child_types,
        owned_parent_fields,
        owned_parent_types,
        linked_children_fields,
        linked_children_types,
        linked_parents_fields,
        linked_parents_types,
        linked_child_fields,
        linked_child_types,
        linked_parent_fields,
        linked_parent_types,
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
    let common_trait = common_node_trait_fns(
        &one_types,
        &many_types,
        &linked_children_types,
        &linked_parents_types,
    );
    let shared_new_fns = shared_new_functions(
        &all_data_fields,
        &all_data_types,
        &one_fields,
        &one_types,
        &many_fields,
        &many_types,
    );
    let owned_child_load = owned_child_fields
        .iter()
        .map(|i| Ident::new(&format!("{i}_load"), Span::call_site()))
        .collect_vec();
    let owned_parent_load = owned_parent_fields
        .iter()
        .map(|i| Ident::new(&format!("{i}_load"), Span::call_site()))
        .collect_vec();
    let owned_children_load = owned_children_fields
        .iter()
        .map(|i| Ident::new(&format!("{i}_load"), Span::call_site()))
        .collect_vec();
    let owned_parents_load = owned_parents_fields
        .iter()
        .map(|i| Ident::new(&format!("{i}_load"), Span::call_site()))
        .collect_vec();
    let owned_child_set = owned_child_fields
        .iter()
        .map(|i| Ident::new(&format!("{i}_set"), Span::call_site()))
        .collect_vec();
    let owned_parent_set = owned_parent_fields
        .iter()
        .map(|i| Ident::new(&format!("{i}_set"), Span::call_site()))
        .collect_vec();
    let owned_children_add = owned_children_fields
        .iter()
        .map(|i| Ident::new(&format!("{i}_add"), Span::call_site()))
        .collect_vec();
    let owned_parents_add = owned_parents_fields
        .iter()
        .map(|i| Ident::new(&format!("{i}_add"), Span::call_site()))
        .collect_vec();

    quote! {
        #[derive(Default, Debug)]
        pub #item
        #common
        #[allow(unused)]
        #[allow(dead_code)]
        #[allow(unused_mut)]
        impl #struct_ident {
            #shared_new_fns
            #(
                pub fn #owned_parent_set(&mut self, ctx: &ReducerContext, mut node: #one_types) -> Result<&mut Self, String> {
                    self.id.add_parent(ctx, node.id)?;
                    self.#owned_parent_fields = Some(node);
                    Ok(self)
                }
            )*
            #(
                pub fn #owned_child_set(&mut self, ctx: &ReducerContext, mut node: #one_types) -> Result<&mut Self, String> {
                    self.id.add_child(ctx, node.id)?;
                    self.#owned_child_fields = Some(node);
                    Ok(self)
                }
            )*
            #(
                pub fn #owned_parents_add(&mut self, ctx: &ReducerContext, mut node: #many_types) -> Result<&mut Self, String> {
                    self.id.add_parent(ctx, node.id)?;
                    self.#owned_parents_fields.push(node);
                    Ok(self)
                }
            )*
            #(
                pub fn #owned_children_add(&mut self, ctx: &ReducerContext, mut node: #many_types) -> Result<&mut Self, String> {
                    self.id.add_child(ctx, node.id)?;
                    self.#owned_children_fields.push(node);
                    Ok(self)
                }
            )*
            #(
                pub fn #owned_child_load<'a>(&'a mut self, ctx: &ReducerContext) -> Result<&'a mut #owned_child_types, String> {
                    let id = self.id();
                    if self.#owned_child_fields.is_none() {
                        self.#owned_child_fields = self.child::<#owned_child_types>(ctx);
                    }
                    self.#owned_child_fields
                        .as_mut()
                        .to_custom_e_s_fn(|| format!("{} not found for {}", #owned_child_types::kind_s(), id))
                }
            )*
            #(
                pub fn #owned_parent_load<'a>(&'a mut self, ctx: &ReducerContext) -> Result<&'a mut #owned_parent_types, String> {
                    let id = self.id();
                    if self.#owned_parent_fields.is_none() {
                        self.#owned_parent_fields = self.parent::<#owned_parent_types>(ctx);
                    }
                    self.#owned_parent_fields
                        .as_mut()
                        .to_custom_e_s_fn(|| format!("{} not found for {}", #owned_parent_types::kind_s(), id))
                }
            )*
            #(
                pub fn #owned_children_load<'a>(&'a mut self, ctx: &ReducerContext) -> Result<&'a mut Vec<#owned_children_types>, String> {
                    if self.#owned_children_fields.is_empty() {
                        self.#owned_children_fields = self.collect_children::<#owned_children_types>(ctx);
                    }
                    if self.#owned_children_fields.is_empty() {
                        return Err(format!("No {} children found for {}", #owned_children_types::kind_s(), self.id()));
                    }
                    Ok(&mut self.#owned_children_fields)
                }
            )*
            #(
                pub fn #owned_parents_load<'a>(&'a mut self, ctx: &ReducerContext) -> Result<&'a mut Vec<#owned_parents_types>, String> {
                    if self.#owned_parents_fields.is_empty() {
                        self.#owned_parents_fields = self.collect_parents::<#owned_parents_types>(ctx);
                    }
                    if self.#owned_parents_fields.is_empty() {
                        return Err(format!("No {} parents found for {}", #owned_parents_types::kind_s(), self.id()));
                    }
                    Ok(&mut self.#owned_parents_fields)
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
        #[allow(unused)]
        #[allow(dead_code)]
        #[allow(unused_mut)]
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
                    owner,
                    #(
                        self.#all_data_fields.clone(),
                    )*
                );
                d.insert_self(ctx);
                d
            }
            fn clone(&self, ctx: &ReducerContext, owner: u64, remap: &mut HashMap<u64, u64>) -> Self {
                let mut d = self.clone_self(ctx, owner);
                remap.insert(self.id, d.id);
                #(
                    if let Some(n) = self.#owned_parent_fields.as_ref() {
                        let n = n.clone(ctx, owner, remap);
                        d.id.add_parent(ctx, n.id).unwrap();
                        d.#owned_parent_fields = Some(n);
                    }
                )*
                #(
                    if let Some(n) = self.#owned_child_fields.as_ref() {
                        let n = n.clone(ctx, owner, remap);
                        d.id.add_child(ctx, n.id).unwrap();
                        d.#owned_child_fields = Some(n);
                    }
                )*
                #(
                    for n in &self.#owned_parents_fields {
                        let n = n.clone(ctx, owner, remap);
                        d.id.add_parent(ctx, n.id).unwrap();
                        d.#owned_parents_fields.push(n);
                    }
                )*
                #(
                    for n in &self.#owned_children_fields {
                        let n = n.clone(ctx, owner, remap);
                        d.id.add_child(ctx, n.id).unwrap();
                        d.#owned_children_fields.push(n);
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
                    if let Some(n) = &self.#owned_parent_fields {
                        TNodeLink::solidify(ctx, n.id, self.id)?;
                        n.solidify_links(ctx)?;
                    }
                )*
                #(
                    if let Some(n) = &self.#owned_child_fields {
                        TNodeLink::solidify(ctx, self.id, n.id)?;
                        n.solidify_links(ctx)?;
                    }
                )*
                #(
                    for n in &self.#owned_children_fields {
                        TNodeLink::solidify(ctx, self.id, n.id)?;
                        n.solidify_links(ctx)?;
                    }
                )*
                #(
                    for n in &self.#owned_parents_fields {
                        TNodeLink::solidify(ctx, n.id, self.id)?;
                        n.solidify_links(ctx)?;
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
