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
        use node_loaders::*;

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
            fn all_linked_parents(self) -> HashSet<NodeKind>;
            fn all_linked_children(self) -> HashSet<NodeKind>;
            fn convert(self, data: &str) -> Result<TNode, ExpressionError>;
            fn delete_with_parts(self, ctx: &ReducerContext, id: u64) -> Result<(), String>;
        }
        impl ServerNodeKind for NodeKind {
            fn all_linked_children(self) -> HashSet<NodeKind> {
                match self {
                    NodeKind::None => HashSet::new(),
                    #(
                        Self::#names => #names::all_linked_children(),
                    )*
                }
            }
            fn all_linked_parents(self) -> HashSet<NodeKind> {
                match self {
                    NodeKind::None => HashSet::new(),
                    #(
                        Self::#names => #names::all_linked_parents(),
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
            fn delete_with_parts(self, ctx: &ReducerContext, id: u64) -> Result<(), String> {
                match self {
                    Self::None => unreachable!(),
                    #(
                        Self::#names => {
                            #names::get(ctx, id).to_custom_e_s_fn(|| format!("Failed to get {self}#{id}"))?.delete_with_parts(ctx);
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
    let loader_ident = quote::format_ident!("{}Loader", struct_ident);
    let pnf = parse_node_fields(&item.fields);

    let ParsedNodeFields {
        var_fields: _,
        var_types: _,
        data_fields: _,
        data_types: _,
        all_data_fields,
        all_data_types,
        children_fields,
        children_types,
        parents_fields,
        parents_types,
        child_fields,
        child_types,
        parent_fields,
        parent_types,
    } = &pnf;
    let (one_fields, one_types) = pnf.one_owned();
    let (many_fields, many_types) = pnf.many_owned();

    let strings_conversions = strings_conversions(
        children_fields,
        children_types,
        parents_fields,
        parents_types,
        child_fields,
        child_types,
        parent_fields,
        parent_types,
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
    let common = common_node_fns(struct_ident, &all_data_fields, &all_data_types);
    let common_trait =
        common_node_trait_fns(children_types, parents_types, child_types, parent_types);
    let shared_new_fns = shared_new_functions(
        &all_data_fields,
        &all_data_types,
        &one_fields,
        &one_types,
        &many_fields,
        &many_types,
    );
    let child_load = child_fields
        .iter()
        .map(|i| Ident::new(&format!("{i}_load"), Span::call_site()))
        .collect_vec();
    let parent_load = parent_fields
        .iter()
        .map(|i| Ident::new(&format!("{i}_load"), Span::call_site()))
        .collect_vec();
    let children_load = children_fields
        .iter()
        .map(|i| Ident::new(&format!("{i}_load"), Span::call_site()))
        .collect_vec();
    let parents_load = parents_fields
        .iter()
        .map(|i| Ident::new(&format!("{i}_load"), Span::call_site()))
        .collect_vec();
    let child_set = child_fields
        .iter()
        .map(|i| Ident::new(&format!("{i}_set"), Span::call_site()))
        .collect_vec();
    let parent_set = parent_fields
        .iter()
        .map(|i| Ident::new(&format!("{i}_set"), Span::call_site()))
        .collect_vec();
    let children_add = children_fields
        .iter()
        .map(|i| Ident::new(&format!("{i}_add"), Span::call_site()))
        .collect_vec();
    let parents_add = parents_fields
        .iter()
        .map(|i| Ident::new(&format!("{i}_add"), Span::call_site()))
        .collect_vec();

    quote! {
        impl ServerLoader<#struct_ident> for #loader_ident {
            fn load(self, ctx: &ReducerContext) -> Result<#struct_ident, String> {
                #struct_ident::load_with(self, ctx)
            }
        }
        #[derive(Default, Debug)]
        pub #item
        #common
        #[allow(unused)]
        #[allow(dead_code)]
        #[allow(unused_mut)]
        impl #struct_ident {
            #shared_new_fns
            #(
                pub fn #parent_set(&mut self, ctx: &ReducerContext, mut node: #one_types) -> Result<&mut Self, String> {
                    self.id.add_parent(ctx, node.id)?;
                    self.#parent_fields.set_data(node);
                    Ok(self)
                }
            )*
            #(
                pub fn #child_set(&mut self, ctx: &ReducerContext, mut node: #one_types) -> Result<&mut Self, String> {
                    self.id.add_child(ctx, node.id)?;
                    self.#child_fields.set_data(node);
                    Ok(self)
                }
            )*
            #(
                pub fn #parents_add(&mut self, ctx: &ReducerContext, mut node: #many_types) -> Result<&mut Self, String> {
                    self.id.add_parent(ctx, node.id)?;
                    self.#parents_fields.push(node);
                    Ok(self)
                }
            )*
            #(
                pub fn #children_add(&mut self, ctx: &ReducerContext, mut node: #many_types) -> Result<&mut Self, String> {
                    self.id.add_child(ctx, node.id)?;
                    self.#children_fields.push(node);
                    Ok(self)
                }
            )*
            #(
                pub fn #child_load<'a>(&'a mut self, ctx: &ReducerContext) -> Result<&'a mut #child_types, String> {
                    let id = self.id();
                    if !self.#child_fields.is_loaded() {
                        if let Some(node) = self.child::<#child_types>(ctx) {
                            self.#child_fields.set_data(node);
                        }
                    }
                    self.#child_fields
                        .get_data_mut()
                        .to_custom_e_s_fn(|| format!("{} not found for {}", #child_types::kind_s(), id))
                }
            )*
            #(
                pub fn #parent_load<'a>(&'a mut self, ctx: &ReducerContext) -> Result<&'a mut #parent_types, String> {
                    let id = self.id();
                    if !self.#parent_fields.is_loaded() {
                        if let Some(node) = self.parent::<#parent_types>(ctx) {
                            self.#parent_fields.set_data(node);
                        }
                    }
                    self.#parent_fields
                        .get_data_mut()
                        .to_custom_e_s_fn(|| format!("{} not found for {}", #parent_types::kind_s(), id))
                }
            )*
            #(
                pub fn #children_load<'a>(&'a mut self, ctx: &ReducerContext) -> Result<&'a mut Vec<#children_types>, String> {
                    if !self.#children_fields.is_loaded() {
                        let children = self.collect_children::<#children_types>(ctx);
                        self.#children_fields.set_data(children);
                    }
                    if self.#children_fields.is_empty() {
                        return Err(format!("No {} children found for {}", #children_types::kind_s(), self.id()));
                    }
                    let id = self.id();
                    self.#children_fields
                        .get_data_mut()
                        .to_custom_e_s_fn(|| format!("{} children not loaded for {}", #children_types::kind_s(), id))
                }
            )*
            #(
                pub fn #parents_load<'a>(&'a mut self, ctx: &ReducerContext) -> Result<&'a mut Vec<#parents_types>, String> {
                    if !self.#parents_fields.is_loaded() {
                        let parents = self.collect_parents::<#parents_types>(ctx);
                        self.#parents_fields.set_data(parents);
                    }
                    if self.#parents_fields.is_empty() {
                        return Err(format!("No {} parents found for {}", #parents_types::kind_s(), self.id()));
                    }
                    let id = self.id();
                    self.#parents_fields
                        .get_data_mut()
                        .to_custom_e_s_fn(|| format!("{} parents not loaded for {}", #parents_types::kind_s(), id))
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

            pub fn loader(id: u64) -> node_loaders::#loader_ident {
                node_loaders::#loader_ident::new(id)
            }

            pub fn load_with(loader: node_loaders::#loader_ident, ctx: &ReducerContext) -> Result<Self, String> {
                let mut node = #struct_ident::get(ctx, loader.id)
                    .ok_or_else(|| format!("{} with id {} not found", stringify!(#struct_ident), loader.id))?;
                #(
                    if loader.#parent_load {
                        let _ = node.#parent_load(ctx);
                    }
                )*
                #(
                    if loader.#child_load {
                        let _ = node.#child_load(ctx);
                    }
                )*
                #(
                    if loader.#parents_load {
                        let _ = node.#parents_load(ctx);
                    }
                )*
                #(
                    if loader.#children_load {
                        let _ = node.#children_load(ctx);
                    }
                )*
                Ok(node)
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
                d.insert(ctx)
            }
            fn clone(&self, ctx: &ReducerContext, owner: u64) -> Self {
                let mut d = self.clone_self(ctx, owner);
                #(
                    if let Some(n) = self.#parent_fields.get_data() {
                        let n = n.clone(ctx, owner);
                        d.id.add_parent(ctx, n.id).unwrap();
                        d.#parent_fields.set_data(n);
                    }
                )*
                #(
                    if let Some(n) = self.#child_fields.get_data() {
                        let n = n.clone(ctx, owner);
                        d.id.add_child(ctx, n.id).unwrap();
                        d.#child_fields.set_data(n);
                    }
                )*
                #(
                    if let Some(parents_data) = self.#parents_fields.get_data() {
                        for n in parents_data {
                            let n = n.clone(ctx, owner);
                            d.id.add_parent(ctx, n.id).unwrap();
                            d.#parents_fields.push(n);
                        }
                    }
                )*
                #(
                    if let Some(children_data) = self.#children_fields.get_data() {
                        for n in children_data {
                            let n = n.clone(ctx, owner);
                            d.id.add_child(ctx, n.id).unwrap();
                            d.#children_fields.push(n);
                        }
                    }
                )*
                d
            }
            fn collect_ids(&self) -> Vec<u64> {
                let mut v = [self.id].to_vec();
                #(
                    if let Some(n) = self.#one_fields.get_data() {
                        v.extend(n.collect_ids());
                    }
                )*
                #(
                    if let Some(many_data) = self.#many_fields.get_data() {
                        for n in many_data {
                            v.extend(n.collect_ids());
                        }
                    }
                )*
                v
            }
            fn solidify_links(&self, ctx: &ReducerContext) -> Result<(), String> {
                #(
                    if let Some(n) = self.#parent_fields.get_data() {
                        TNodeLink::solidify(ctx, n.id, self.id)?;
                        n.solidify_links(ctx)?;
                    }
                )*
                #(
                    if let Some(n) = self.#child_fields.get_data() {
                        TNodeLink::solidify(ctx, self.id, n.id)?;
                        n.solidify_links(ctx)?;
                    }
                )*


                #(
                    if let Some(children_data) = self.#children_fields.get_data() {
                        for n in children_data {
                            TNodeLink::solidify(ctx, self.id, n.id)?;
                            n.solidify_links(ctx)?;
                        }
                    }
                )*
                #(
                    if let Some(parents_data) = self.#parents_fields.get_data() {
                        for n in parents_data {
                            TNodeLink::solidify(ctx, n.id, self.id)?;
                            n.solidify_links(ctx)?;
                        }
                    }
                )*


                Ok(())
            }
            fn with_parts(&mut self, ctx: &ReducerContext) -> &mut Self {
                #(
                    if let Some(mut parent) = self.parent::<#parent_types>(ctx) {
                        self.#parent_fields.set_data(std::mem::take(parent.with_parts(ctx)));
                    }
                )*
                #(
                    if let Some(mut child) = self.child::<#child_types>(ctx) {
                        self.#child_fields.set_data(std::mem::take(child.with_parts(ctx)));
                    }
                )*
                #(
                    let parents_data = self.collect_parents::<#parents_types>(ctx)
                        .into_iter()
                        .map(|mut n| std::mem::take(n.with_parts(ctx)))
                        .collect();
                    self.#parents_fields.set_data(parents_data);
                )*
                #(
                    let children_data = self.collect_children::<#children_types>(ctx)
                        .into_iter()
                        .map(|mut n| std::mem::take(n.with_parts(ctx)))
                        .collect();
                    self.#children_fields.set_data(children_data);
                )*
                self
            }
            fn save(&self, ctx: &ReducerContext) {
                self.update(ctx);
                #(
                    if let Some(d) = self.#one_fields.get_data() {
                        d.save(ctx);
                    }
                )*
                #(
                    if let Some(many_data) = self.#many_fields.get_data() {
                        for d in many_data {
                            d.save(ctx);
                        }
                    }
                )*
            }
            fn delete_with_parts(&self, ctx: &ReducerContext) {
                #(
                    if let Some(p) = self.parent::<#parent_types>(ctx) {
                        p.delete_with_parts(ctx);
                    }
                )*
                #(
                    if let Some(c) = self.child::<#child_types>(ctx) {
                        c.delete_with_parts(ctx);
                    }
                )*
                #(
                    for p in self.collect_parents::<#parents_types>(ctx) {
                        p.delete_with_parts(ctx);
                    }
                )*
                #(
                    for c in self.collect_children::<#children_types>(ctx) {
                        c.delete_with_parts(ctx);
                    }
                )*
                self.delete(ctx);
            }
        }
    }
    .into()
}
