use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use std::collections::{HashMap, HashSet};
use strum_macros::AsRefStr;
use syn::*;

// Function to generate comprehensive allow attributes for all generated code
pub fn generated_code_allow_attrs() -> proc_macro2::TokenStream {
    quote! {
        #[allow(
            dead_code,
            unused_variables,
            unused_mut,
            unreachable_code,
            unused_assignments,
            unused_imports,
            non_snake_case,
            non_camel_case_types
        )]
    }
}

#[derive(Debug, Clone)]
pub struct NodeInfo {
    pub name: Ident,
    pub is_content: bool,
    pub is_named: bool,
    pub name_field: Option<Ident>,
    pub fields: Vec<FieldInfo>,
}

#[derive(Debug, Clone)]
pub struct FieldInfo {
    pub name: Ident,
    pub link_type: LinkType,
    pub target_type: String,
    pub raw_type: String,
    pub is_var: bool,
    pub is_many_to_one: bool,
}

#[derive(Debug, Clone, PartialEq, AsRefStr)]
pub enum LinkType {
    Component,
    Owned,
    OwnedMultiple,
    Ref,
    RefMultiple,
    None,
}

pub fn parse_nodes_file(input: &str) -> syn::Result<Vec<NodeInfo>> {
    let syntax_tree = parse_file(input)?;
    let mut nodes = Vec::new();

    for item in syntax_tree.items {
        if let Item::Struct(item_struct) = item {
            if has_node_attribute(&item_struct) {
                nodes.push(parse_node(&item_struct));
            }
        }
    }

    Ok(nodes)
}

pub fn has_node_attribute(item_struct: &ItemStruct) -> bool {
    item_struct.attrs.iter().any(|attr| {
        if attr.path().is_ident("derive") {
            if let Ok(syn::Meta::List(meta_list)) = attr.meta.clone().try_into() {
                for token in meta_list.tokens.clone() {
                    if token.to_string().contains("Node") {
                        return true;
                    }
                }
            }
        }
        false
    })
}

pub fn has_content_attribute(item_struct: &ItemStruct) -> bool {
    item_struct
        .attrs
        .iter()
        .any(|attr| attr.path().is_ident("content"))
}

pub fn has_named_attribute(item_struct: &ItemStruct) -> bool {
    item_struct
        .attrs
        .iter()
        .any(|attr| attr.path().is_ident("named"))
}

pub fn parse_named_attribute(item_struct: &ItemStruct) -> Option<Ident> {
    for attr in &item_struct.attrs {
        if attr.path().is_ident("named") {
            if let Ok(meta) = attr.parse_args::<Ident>() {
                return Some(meta);
            }
        }
    }
    None
}

pub fn parse_node(item_struct: &ItemStruct) -> NodeInfo {
    let name = item_struct.ident.clone();
    let is_content = has_content_attribute(item_struct);
    let is_named = has_named_attribute(item_struct);
    let name_field = parse_named_attribute(item_struct);

    let fields = item_struct
        .fields
        .iter()
        .map(|field| parse_field(field))
        .collect();

    NodeInfo {
        name,
        is_content,
        is_named,
        name_field,
        fields,
    }
}

pub fn parse_field(field: &Field) -> FieldInfo {
    let name = field.ident.clone().unwrap();
    let (link_type, target_type) = parse_field_type(&field.ty);

    let raw_type = match &field.ty {
        Type::Path(type_path) => quote! { #type_path }.to_string(),
        _ => String::new(),
    };

    let is_var = field.attrs.iter().any(|attr| attr.path().is_ident("var"));
    let is_many_to_one = field
        .attrs
        .iter()
        .any(|attr| attr.path().is_ident("many_to_one"));

    FieldInfo {
        name,
        link_type,
        target_type,
        raw_type,
        is_var,
        is_many_to_one,
    }
}

pub fn parse_field_type(ty: &Type) -> (LinkType, String) {
    if let Type::Path(type_path) = ty {
        let path = &type_path.path;

        if let Some(segment) = path.segments.last() {
            match segment.ident.to_string().as_str() {
                "Component" => {
                    let target = parse_generic_arg(&segment.arguments);
                    return (LinkType::Component, target);
                }
                "Owned" => {
                    let target = parse_generic_arg(&segment.arguments);
                    return (LinkType::Owned, target);
                }
                "OwnedMultiple" => {
                    let target = parse_generic_arg(&segment.arguments);
                    return (LinkType::OwnedMultiple, target);
                }
                "Ref" => {
                    let target = parse_generic_arg(&segment.arguments);
                    return (LinkType::Ref, target);
                }
                "RefMultiple" => {
                    let target = parse_generic_arg(&segment.arguments);
                    return (LinkType::RefMultiple, target);
                }
                _ => {}
            }
        }
    }

    (LinkType::None, String::new())
}

pub fn parse_generic_arg(arguments: &PathArguments) -> String {
    if let PathArguments::AngleBracketed(args) = arguments {
        if let Some(GenericArgument::Type(ty)) = args.args.first() {
            if let Type::Path(type_path) = ty {
                if let Some(segment) = type_path.path.segments.last() {
                    let type_name = segment.ident.to_string();
                    // Check if it's a Vec, which is not allowed
                    if type_name == "Vec" {
                        panic!("Vec<...> is not supported in link types. Use Multiple variants instead.");
                    }
                    // Validate that it's a node type (starts with 'N')
                    if !type_name.starts_with('N') {
                        panic!(
                            "Link type target must be a Node type (starting with 'N'), found: {}",
                            type_name
                        );
                    }
                    return type_name;
                }
            }
        }
    }
    panic!()
}

pub fn validate_parent_relationships(
    node_map: &HashMap<String, NodeInfo>,
) -> std::result::Result<(), String> {
    // Only validate content nodes for single content parent restriction
    let mut content_parent_map: HashMap<String, Vec<String>> = HashMap::new();

    for (node_name, node_info) in node_map {
        if !node_info.is_content {
            continue;
        }

        for field in &node_info.fields {
            if matches!(
                field.link_type,
                LinkType::Component | LinkType::Owned | LinkType::OwnedMultiple
            ) {
                // Check if the target is also a content node
                if let Some(target_info) = node_map.get(&field.target_type) {
                    if target_info.is_content {
                        content_parent_map
                            .entry(field.target_type.clone())
                            .or_insert_with(Vec::new)
                            .push(node_name.clone());
                    }
                }
            }
        }
    }

    // Check that content nodes only have one content parent
    for (child_name, parents) in &content_parent_map {
        if parents.len() > 1 {
            return Err(format!(
                "Content node {} has multiple content parents: {:?}. A content node can only have one content parent.",
                child_name, parents
            ));
        }
    }

    Ok(())
}

pub struct RelationshipMaps {
    pub component_parents: HashMap<String, String>,
    pub component_children: HashMap<String, HashSet<String>>,
    pub owned_parents: HashMap<String, String>,
    pub owned_children: HashMap<String, HashSet<String>>,
}

pub fn build_relationship_maps(node_map: &HashMap<String, NodeInfo>) -> RelationshipMaps {
    let mut component_parents: HashMap<String, String> = HashMap::new();
    let mut component_children: HashMap<String, HashSet<String>> = HashMap::new();
    let mut owned_parents: HashMap<String, String> = HashMap::new();
    let mut owned_children: HashMap<String, HashSet<String>> = HashMap::new();

    for (node_name, node_info) in node_map {
        for field in &node_info.fields {
            match field.link_type {
                LinkType::Component => {
                    component_parents.insert(field.target_type.clone(), node_name.clone());
                    component_children
                        .entry(node_name.clone())
                        .or_insert_with(HashSet::new)
                        .insert(field.target_type.clone());
                }
                LinkType::Owned | LinkType::OwnedMultiple => {
                    owned_parents.insert(field.target_type.clone(), node_name.clone());
                    owned_children
                        .entry(node_name.clone())
                        .or_insert_with(HashSet::new)
                        .insert(field.target_type.clone());
                }
                _ => {}
            }
        }
    }

    RelationshipMaps {
        component_parents,
        component_children,
        owned_parents,
        owned_children,
    }
}

pub fn generate_parent_arms(parents: &HashMap<String, String>) -> TokenStream {
    let arms: Vec<TokenStream> = parents
        .iter()
        .map(|(child, parent)| {
            let child_ident = format_ident!("{}", child);
            let parent_ident = format_ident!("{}", parent);
            quote! {
                NodeKind::#child_ident => Some(NodeKind::#parent_ident),
            }
        })
        .collect();

    quote! { #(#arms)* }
}

pub fn generate_children_arms(children: &HashMap<String, HashSet<String>>) -> TokenStream {
    let arms: Vec<TokenStream> = children
        .iter()
        .map(|(parent, child_set)| {
            let parent_ident = format_ident!("{}", parent);
            let child_idents: Vec<TokenStream> = child_set
                .iter()
                .map(|child| {
                    let child_ident = format_ident!("{}", child);
                    quote! { set.insert(NodeKind::#child_ident); }
                })
                .collect();

            quote! {
                NodeKind::#parent_ident => {
                    let mut set = HashSet::new();
                    #(#child_idents)*
                    set
                },
            }
        })
        .collect();

    quote! { #(#arms)* }
}

pub fn generate_children_recursive_arms(
    children: &HashMap<String, HashSet<String>>,
) -> TokenStream {
    let arms: Vec<TokenStream> = children
        .iter()
        .map(|(parent, _)| {
            let parent_ident = format_ident!("{}", parent);
            quote! {
                NodeKind::#parent_ident => {
                    let mut result = HashSet::new();
                    let mut to_visit = vec![self];
                    let mut visited = HashSet::new();

                    while let Some(current) = to_visit.pop() {
                        if !visited.insert(current) {
                            continue;
                        }

                        for child in current.component_children() {
                            result.insert(child);
                            to_visit.push(child);
                        }
                    }

                    result
                },
            }
        })
        .collect();

    quote! { #(#arms)* }
}

pub fn generate_owning_parent_arms(
    owned_parents: &HashMap<String, String>,
) -> proc_macro2::TokenStream {
    let arms = owned_parents.iter().map(|(child, parent)| {
        let child_ident = format_ident!("{}", child);
        let parent_ident = format_ident!("{}", parent);
        quote! {
            NodeKind::#child_ident => { parents.push(NodeKind::#parent_ident); }
        }
    });
    quote! { #(#arms)* }
}

pub fn generate_owning_children_arms(
    owned_children: &HashMap<String, HashSet<String>>,
) -> proc_macro2::TokenStream {
    let arms = owned_children.iter().map(|(parent, children)| {
        let parent_ident = format_ident!("{}", parent);
        let child_idents: Vec<_> = children
            .iter()
            .map(|child| format_ident!("{}", child))
            .collect();
        quote! {
            NodeKind::#parent_ident => {
                #(children.push(NodeKind::#child_idents);)*
            }
        }
    });
    quote! { #(#arms)* }
}

pub fn generate_other_components_arms(
    component_parents: &HashMap<String, String>,
    component_children: &HashMap<String, HashSet<String>>,
) -> TokenStream {
    let mut component_groups: HashMap<String, HashSet<String>> = HashMap::new();

    for node in component_parents.keys().chain(component_children.keys()) {
        let mut group = HashSet::new();
        let mut to_visit = vec![node.clone()];
        let mut visited = HashSet::new();

        while let Some(current) = to_visit.pop() {
            if visited.contains(&current) {
                continue;
            }
            visited.insert(current.clone());
            group.insert(current.clone());

            if let Some(parent) = component_parents.get(&current) {
                to_visit.push(parent.clone());
            }

            if let Some(children) = component_children.get(&current) {
                for child in children {
                    to_visit.push(child.clone());
                }
            }
        }

        group.remove(node);
        component_groups.insert(node.clone(), group);
    }

    let arms: Vec<TokenStream> = component_groups
        .iter()
        .map(|(node, others)| {
            let node_ident = format_ident!("{}", node);
            let other_idents: Vec<TokenStream> = others
                .iter()
                .map(|other| {
                    let other_ident = format_ident!("{}", other);
                    quote! { set.insert(NodeKind::#other_ident); }
                })
                .collect();

            quote! {
                NodeKind::#node_ident => {
                    let mut set = HashSet::new();
                    #(#other_idents)*
                    set
                },
            }
        })
        .collect();

    quote! { #(#arms)* }
}

pub fn format_code(token_stream: &TokenStream) -> String {
    match syn::parse_file(&token_stream.to_string()) {
        Ok(parsed) => prettyplease::unparse(&parsed),
        Err(_) => token_stream.to_string(),
    }
}

pub fn generate_field_type(field: &FieldInfo) -> TokenStream {
    match field.link_type {
        LinkType::Component => {
            let target = if field.target_type.is_empty() {
                quote! { String }
            } else {
                let target_ident = format_ident!("{}", field.target_type);
                quote! { #target_ident }
            };
            quote! { Component<#target> }
        }

        LinkType::Owned => {
            let target = if field.target_type.is_empty() {
                quote! { String }
            } else {
                let target_ident = format_ident!("{}", field.target_type);
                quote! { #target_ident }
            };
            quote! { Owned<#target> }
        }
        LinkType::OwnedMultiple => {
            let target = if field.target_type.is_empty() {
                quote! { String }
            } else {
                let target_ident = format_ident!("{}", field.target_type);
                quote! { #target_ident }
            };
            quote! { OwnedMultiple<#target> }
        }
        LinkType::Ref => {
            let target = if field.target_type.is_empty() {
                quote! { String }
            } else {
                let target_ident = format_ident!("{}", field.target_type);
                quote! { #target_ident }
            };
            quote! { Ref<#target> }
        }
        LinkType::RefMultiple => {
            let target = if field.target_type.is_empty() {
                quote! { String }
            } else {
                let target_ident = format_ident!("{}", field.target_type);
                quote! { #target_ident }
            };
            quote! { RefMultiple<#target> }
        }
        LinkType::None => {
            // For primitive types, use the raw type directly
            if field.raw_type.is_empty() {
                quote! { String }
            } else {
                let tokens: TokenStream =
                    field.raw_type.parse().unwrap_or_else(|_| quote! { String });
                tokens
            }
        }
    }
}

pub fn generate_new(node: &NodeInfo) -> TokenStream {
    let data_fields: Vec<_> = node
        .fields
        .iter()
        .filter(|field| field.link_type == LinkType::None)
        .collect();

    let params = std::iter::once(quote! { node_id: u64 })
        .chain(std::iter::once(quote! { owner_id: u64 }))
        .chain(data_fields.iter().map(|field| {
            let field_name = &field.name;
            let field_type = generate_field_type(field);
            quote! { #field_name: #field_type }
        }));

    let field_assignments = data_fields.iter().map(|field| {
        let field_name = &field.name;
        quote! { #field_name, }
    });

    let component_defaults = node
        .fields
        .iter()
        .filter(|field| field.link_type != LinkType::None)
        .map(|field| {
            let field_name = &field.name;
            let link_type = Ident::new(field.link_type.as_ref(), Span::call_site());
            quote! { #field_name: #link_type::unknown(node_id), }
        });

    quote! {
        pub fn new(#(#params),*) -> Self {
            Self {
                id: node_id,
                owner: owner_id,
                #(#field_assignments)*
                #(#component_defaults)*
            }
        }
    }
}

pub fn generate_with_methods(node: &NodeInfo) -> TokenStream {
    let methods = node.fields.iter().filter_map(|field| {
        if matches!(field.link_type, LinkType::None) {
            return None;
        }

        let field_name = &field.name;
        let with_method = format_ident!("with_{}", field_name);
        let clear_method = format_ident!("{}_clear", field_name);

        let target = if field.target_type.is_empty() {
            quote! { String }
        } else {
            let target_ident = format_ident!("{}", field.target_type);
            quote! { #target_ident }
        };

        let (param_type, wrapped_value, clear_value) = match field.link_type {
            LinkType::Component => (
                quote! { #target },
                quote! { Component::new_loaded(self.id, value) },
                quote! { Component::none(self.id) },
            ),
            LinkType::Owned => (
                quote! { #target },
                quote! { Owned::new_loaded(self.id, value) },
                quote! { Owned::none(self.id) },
            ),
            LinkType::OwnedMultiple => (
                quote! { Vec<#target> },
                quote! { OwnedMultiple::new_loaded(self.id, value) },
                quote! { OwnedMultiple::none(self.id) },
            ),
            LinkType::Ref => (
                quote! { #target },
                quote! { Ref::new_loaded(self.id, value) },
                quote! { Ref::none(self.id) },
            ),
            LinkType::RefMultiple => (
                quote! { Vec<#target> },
                quote! { RefMultiple::new_loaded(self.id, value) },
                quote! { RefMultiple::none(self.id) },
            ),
            _ => return None,
        };

        Some(quote! {
            pub fn #with_method(mut self, value: #param_type) -> Self {
                self.#field_name = #wrapped_value;
                self
            }

            pub fn #clear_method(mut self) -> Self {
                self.#field_name = #clear_value;
                self
            }
        })
    });

    quote! {
        #(#methods)*
    }
}

pub fn generate_default_impl(node: &NodeInfo) -> TokenStream {
    let struct_name = &node.name;
    let fields = node.fields.iter().map(|f| f.name.clone());

    quote! {
        impl Default for #struct_name {
            fn default() -> Self {
                Self {
                    id: 0,
                    owner: 0,
                    #(#fields: default(),)*
                }
            }
        }
    }
}

// Generate simple link accessor methods for node fields
pub fn generate_link_accessor_methods(node: &NodeInfo) -> TokenStream {
    let methods = node.fields.iter().filter_map(|field| {
        if matches!(field.link_type, LinkType::None) {
            return None;
        }

        let field_name = &field.name;
        let target_type = format_ident!("{}", field.target_type);

        // Generate method names
        let get_method = format_ident!("{}", field_name);
        let get_mut_method = format_ident!("{}_mut", field_name);
        let set_method = format_ident!("{}_set", field_name);
        let clear_method = format_ident!("{}_clear", field_name);

        match field.link_type {
            LinkType::Component | LinkType::Owned | LinkType::Ref => Some(quote! {
                pub fn #get_method(&self) -> NodeResult<&#target_type> {
                    self.#field_name.get()
                }

                pub fn #get_mut_method(&mut self) -> NodeResult<&mut #target_type> {
                    self.#field_name.get_mut()
                }

                pub fn #set_method(&mut self, value: #target_type) -> NodeResult<()> {
                    self.#field_name.set_loaded(value)
                }

                pub fn #clear_method(&mut self) -> NodeResult<()> {
                    self.#field_name.set_none()?;
                    Ok(())
                }
            }),
            LinkType::OwnedMultiple | LinkType::RefMultiple => Some(quote! {
                pub fn #get_method(&self) -> NodeResult<&Vec<#target_type>> {
                    self.#field_name.get()
                }

                pub fn #get_mut_method(&mut self) -> NodeResult<&mut Vec<#target_type>> {
                    self.#field_name.get_mut()
                }

                pub fn #set_method(&mut self, value: Vec<#target_type>) -> NodeResult<()> {
                    self.#field_name.set_loaded(value)
                }

                pub fn #clear_method(&mut self) -> NodeResult<()> {
                    self.#field_name.set_none()
                }
            }),
            _ => None,
        }
    });

    quote! {
        #(#methods)*
    }
}

pub fn generate_node_impl(nodes: &[NodeInfo]) -> TokenStream {
    let node_trait_impls = nodes.iter().map(|node| {
        let struct_name = &node.name;
        let node_kind_variant = &node.name;
        let pack_links_impl = generate_pack_links_impl(node);
        let unpack_links_impl = generate_unpack_links_impl(node);
        let var_methods = generate_var_methods(node);
        let var_accessor_methods = generate_var_accessor_methods(node);

        // Generate collect methods
        let collect_owned_ids_method = generate_collect_owned_ids_impl(node);
        let collect_owned_links_method = generate_collect_owned_links_impl(node);

        let update_link_references_impl = generate_update_link_references_impl(node);
        let set_owner_calls = generate_set_owner_calls(node);
        let allow_attrs = generated_code_allow_attrs();

        quote! {
            #allow_attrs
            impl Node for #struct_name {
                fn id(&self) -> u64 {
                    self.id
                }

                fn set_id(&mut self, id: u64) {
                    self.id = id;
                }

                fn owner(&self) -> u64 {
                    self.owner
                }

                fn set_owner(&mut self, owner: u64) {
                    self.owner = owner;

                    // Update owner for loaded linked nodes (Owned and Component types)
                    #(#set_owner_calls)*
                }



                fn reassign_ids(&mut self, next_id: &mut u64, id_map: &mut std::collections::HashMap<u64, u64>) {
                    // Record old ID and assign new ID
                    let old_id = self.id();
                    let new_id = *next_id;
                    self.set_id(new_id);
                    *next_id += 1;
                    id_map.insert(old_id, new_id);

                    // Phase 1: Recursively reassign IDs for owned children
                    self.reassign_owned_ids(next_id, id_map);

                    // Phase 2: Update reference links using the populated id_map
                    self.update_reference_links(id_map);
                }

                fn kind_s() -> NodeKind {
                    NodeKind::#node_kind_variant
                }





                #var_methods

                #pack_links_impl
                #unpack_links_impl

                #collect_owned_ids_method
                #collect_owned_links_method
            }

            #allow_attrs
            impl #struct_name {
                #update_link_references_impl
            }

            #var_accessor_methods
        }
    });

    quote! {
        #(#node_trait_impls)*
    }
}

/// Generate link loading methods for nodes with specified context type

pub fn generate_load_functions(node: &NodeInfo, context_ident: &str) -> TokenStream {
    let context_ident = Ident::new(context_ident, proc_macro2::Span::call_site());
    let component_fields: Vec<_> = node
        .fields
        .iter()
        .filter(|field| matches!(field.link_type, LinkType::Component))
        .collect();

    let owned_fields: Vec<_> = node
        .fields
        .iter()
        .filter(|field| matches!(field.link_type, LinkType::Owned | LinkType::OwnedMultiple))
        .collect();

    // Generate load_components method (only Component links, recursive)
    let component_field_loads = component_fields.iter().map(|field| {
        let field_name = &field.name;

        quote! {
            if let Ok(loaded_item) = self.#field_name.get_mut() {
                loaded_item.load_components(ctx)?;
            }
        }
    });

    let load_components_method = if component_fields.is_empty() {
        quote! {
            fn load_components(&mut self, _ctx: &#context_ident) -> NodeResult<&mut Self> {
                Ok(self)
            }
        }
    } else {
        quote! {
            fn load_components(&mut self, ctx: &#context_ident) -> NodeResult<&mut Self> {
                #(#component_field_loads)*
                Ok(self)
            }
        }
    };

    // Generate load_all method (Component + Owned + Ref links)
    let all_field_loads = node
        .fields
        .iter()
        .filter_map(|field| match field.link_type {
            LinkType::Component | LinkType::Owned | LinkType::OwnedMultiple => {
                let field_name = &field.name;

                Some(match field.link_type {
                    LinkType::OwnedMultiple => quote! {
                        if let Ok(loaded_items) = self.#field_name.get_mut() {
                            for item in loaded_items.iter_mut() {
                                item.load_all(ctx)?;
                            }
                        }
                    },
                    _ => quote! {
                        if let Ok(loaded_item) = self.#field_name.get_mut() {
                            loaded_item.load_all(ctx)?;
                        }
                    },
                })
            }
            LinkType::Ref | LinkType::RefMultiple => None,
            LinkType::None => None,
        });

    let ref_fields: Vec<_> = node
        .fields
        .iter()
        .filter(|field| matches!(field.link_type, LinkType::Ref | LinkType::RefMultiple))
        .collect();

    let load_all_method =
        if component_fields.is_empty() && owned_fields.is_empty() && ref_fields.is_empty() {
            quote! {
                fn load_all(&mut self, _ctx: &#context_ident) -> NodeResult<&mut Self> {
                    Ok(self)
                }
            }
        } else {
            quote! {
                fn load_all(&mut self, ctx: &#context_ident) -> NodeResult<&mut Self> {
                    #(#all_field_loads)*
                    Ok(self)
                }
            }
        };

    quote! {
        #load_components_method

        #load_all_method
    }
}

pub fn generate_pack_links_impl(node: &NodeInfo) -> proc_macro2::TokenStream {
    let struct_name = &node.name;

    let pack_link_calls = node.fields.iter().filter_map(|field| {
        if matches!(field.link_type, LinkType::None) {
            return None;
        }

        let field_name = &field.name;
        let target_type = format_ident!("{}", field.target_type);

        Some(match field.link_type {
            LinkType::OwnedMultiple => quote! {
                match &self.#field_name {
                    OwnedMultiple::Loaded(_, items) => {
                        for item in items {
                            item.pack_recursive(packed, visited);
                            packed.link_parent_child(
                                self.id,
                                item.id(),
                                stringify!(#struct_name).to_string(),
                                stringify!(#target_type).to_string()
                            );
                        }
                    }
                    _ => {}
                }
            },
            LinkType::Component => quote! {
                match &self.#field_name {
                    Component::Loaded(_, item) => {
                        item.pack_recursive(packed, visited);
                        packed.link_parent_child(
                            self.id,
                            item.id(),
                            stringify!(#struct_name).to_string(),
                            stringify!(#target_type).to_string()
                        );
                    },
                    _ => {}
                }
            },
            LinkType::Owned => quote! {
                match &self.#field_name {
                    Owned::Loaded(_, item) => {
                        item.pack_recursive(packed, visited);
                        packed.link_parent_child(
                            self.id,
                            item.id(),
                            stringify!(#struct_name).to_string(),
                            stringify!(#target_type).to_string()
                        );
                    },
                    _ => {}
                }
            },
            LinkType::Ref => quote! {
                match &self.#field_name {
                    Ref::Loaded(_, item) => {
                        packed.link_parent_child(
                            self.id,
                            item.id(),
                            stringify!(#struct_name).to_string(),
                            stringify!(#target_type).to_string()
                        );
                    }
                    _ => {}
                }
            },

            LinkType::RefMultiple => quote! {
                match &self.#field_name {
                    RefMultiple::Loaded(_, items) => {
                        for item in items {
                            packed.link_parent_child(
                                self.id,
                                item.id(),
                                stringify!(#struct_name).to_string(),
                                stringify!(#target_type).to_string()
                            );
                        }
                    }
                    _ => {}
                }
            },
            _ => quote! {},
        })
    });

    quote! {
        fn pack_links(&self, packed: &mut PackedNodes, visited: &mut std::collections::HashSet<u64>) {
            #(#pack_link_calls)*
        }
    }
}

pub fn generate_unpack_links_impl(node: &NodeInfo) -> proc_macro2::TokenStream {
    let unpack_link_calls = node.fields.iter().filter_map(|field| {
        if matches!(field.link_type, LinkType::None) {
            return None;
        }

        let field_name = &field.name;
        let target_type = format_ident!("{}", field.target_type);

        Some(match field.link_type {
            LinkType::OwnedMultiple => quote! {
                let child_ids = packed.kind_children(self.id, stringify!(#target_type));
                let mut children = Vec::new();
                for child_id in child_ids {
                    if let Some(child_data) = packed.get(child_id) {
                        let mut child = #target_type::default();
                        child.inject_data(&child_data.data).unwrap();
                        child.set_id(child_id);
                        child.unpack_links(packed);
                        children.push(child);
                    }
                }
                if !children.is_empty() {
                    self.#field_name = OwnedMultiple::new_loaded(self.id, children);
                } else {
                    self.#field_name = OwnedMultiple::none(self.id);
                }
            },
            LinkType::RefMultiple => quote! {
                let child_ids = packed.kind_children(self.id, stringify!(#target_type));
                if !child_ids.is_empty() {
                    let mut children = Vec::new();
                    for child_id in child_ids {
                        if let Some(child_data) = packed.get(child_id) {
                            let mut child = #target_type::default();
                            child.inject_data(&child_data.data).unwrap();
                            child.set_id(child_id);
                            child.unpack_links(packed);
                            children.push(child);
                        }
                    }
                    self.#field_name = RefMultiple::new_loaded(self.id, children);
                } else {
                    self.#field_name = RefMultiple::none(self.id);
                }
            },
            LinkType::Component => quote! {
                let child_ids = packed.kind_children(self.id, stringify!(#target_type));
                if let Some(&child_id) = child_ids.first() {
                    if let Some(child_data) = packed.get(child_id) {
                        let mut child = #target_type::default();
                        child.inject_data(&child_data.data).unwrap();
                        child.set_id(child_id);
                        child.unpack_links(packed);
                        self.#field_name = Component::new_loaded(self.id, child);
                    }
                } else {
                    self.#field_name = Component::none(self.id);
                }
            },
            LinkType::Owned => quote! {
                let child_ids = packed.kind_children(self.id, stringify!(#target_type));
                if let Some(&child_id) = child_ids.first() {
                    if let Some(child_data) = packed.get(child_id) {
                        let mut child = #target_type::default();
                        child.inject_data(&child_data.data).unwrap();
                        child.set_id(child_id);
                        child.unpack_links(packed);
                        self.#field_name = Owned::new_loaded(self.id, child);
                    }
                } else {
                    self.#field_name = Owned::none(self.id);
                }
            },
            LinkType::Ref => quote! {
                let child_ids = packed.kind_children(self.id, stringify!(#target_type));
                if let Some(&child_id) = child_ids.first() {
                    if let Some(child_data) = packed.get(child_id) {
                        let mut child = #target_type::default();
                        child.inject_data(&child_data.data).unwrap();
                        child.set_id(child_id);
                        child.unpack_links(packed);
                        self.#field_name = Ref::new_loaded(self.id, child);
                    }
                } else {
                    self.#field_name = Ref::none(self.id);
                }
            },
            _ => quote! {},
        })
    });

    quote! {
        fn unpack_links(&mut self, packed: &PackedNodes) {
            #(#unpack_link_calls)*
        }
    }
}

pub fn generate_var_methods(node: &NodeInfo) -> proc_macro2::TokenStream {
    let var_fields: Vec<_> = node.fields.iter().filter(|f| f.is_var).collect();

    // Generate var_names method
    let var_names: Vec<_> = var_fields
        .iter()
        .map(|f| {
            let field_name = &f.name;
            quote! { VarName::#field_name }
        })
        .collect();

    let allow_attrs = generated_code_allow_attrs();
    let var_names_impl = quote! {
        fn var_names() -> Vec<VarName>
        where
            Self: Sized,
        {
            vec![#(#var_names),*]
        }
    };

    // Generate get_var method
    let get_var_arms: Vec<_> = var_fields
        .iter()
        .map(|f| {
            let field_name = &f.name;
            quote! {
                VarName::#field_name => Ok(self.#field_name.clone().into())
            }
        })
        .collect();

    let get_var_impl = quote! {
        #allow_attrs
        fn get_var(&self, var: VarName) -> NodeResult<VarValue> {
            match var {
                #(#get_var_arms,)*
                _ => Err(NodeError::custom(format!("Variable {:?} not found", var))),
            }
        }
    };

    // Generate get_vars method
    let get_vars_inserts: Vec<_> = var_fields
        .iter()
        .map(|f| {
            let field_name = &f.name;
            quote! {
                (VarName::#field_name, self.#field_name.clone().into())
            }
        })
        .collect();

    let get_vars_impl = quote! {
        fn get_vars(&self) -> Vec<(VarName, VarValue)> {
            vec![#(#get_vars_inserts),*]
        }
    };

    // Generate set_var method
    let set_var_arms: Vec<_> = var_fields
        .iter()
        .map(|f| {
            let field_name = &f.name;
            quote! {
                VarName::#field_name => {
                    self.#field_name = value.try_into().map_err(|_| NodeError::custom("Value conversion failed"))?;
                    Ok(())
                }
            }
        })
        .collect();

    let set_var_impl = quote! {
        fn set_var(&mut self, var: VarName, value: VarValue) -> NodeResult<()> {
            match var {
                #(#set_var_arms,)*
                _ => Err(NodeError::custom(format!("Tried to set VarName::{:?} that is absent in {}", var, Self::kind_s()))),
            }
        }
    };

    quote! {
        #var_names_impl
        #get_var_impl
        #set_var_impl
        #get_vars_impl
    }
}

pub fn generate_var_accessor_methods(node: &NodeInfo) -> proc_macro2::TokenStream {
    let var_fields: Vec<_> = node.fields.iter().filter(|f| f.is_var).collect();

    if var_fields.is_empty() {
        return quote! {};
    }

    let allow_attrs = generated_code_allow_attrs();
    let struct_name = &node.name;

    let accessor_methods: Vec<_> = var_fields
        .iter()
        .map(|f| {
            let field_name = &f.name;
            let field_type = generate_field_type(f);
            let get_method_name = format_ident!("{}_get", field_name);
            let set_method_name = format_ident!("{}_set", field_name);
            let ctx_get_method_name = format_ident!("{}_ctx_get", field_name);
            let var_name = format_ident!("{}", field_name);

            quote! {
                #allow_attrs
                pub fn #get_method_name(&self) -> #field_type {
                    self.#field_name.clone()
                }

                #allow_attrs
                pub fn #ctx_get_method_name<S: ContextSource>(&self, ctx: &Context<S>) -> #field_type {
                    if let Ok(value) = ctx.source().get_var(self.id(), VarName::#var_name) {
                        return value.into();
                    }
                    self.#field_name.clone()
                }

                #allow_attrs
                pub fn #set_method_name(&mut self, value: #field_type) {
                    self.#field_name = value;
                }
            }
        })
        .collect();

    quote! {
        impl #struct_name {
            #(#accessor_methods)*
        }
    }
}

pub fn generate_var_names_for_node_kind(nodes: &[NodeInfo]) -> proc_macro2::TokenStream {
    let allow_attrs = generated_code_allow_attrs();
    let var_names_arms = nodes.iter().map(|node| {
        let node_name = &node.name;
        let var_fields: Vec<_> = node.fields.iter().filter(|f| f.is_var).collect();
        let var_names: Vec<_> = var_fields
            .iter()
            .map(|f| {
                let field_name = &f.name;
                quote! { VarName::#field_name }
            })
            .collect();

        quote! {
            NodeKind::#node_name => {
                vec![#(#var_names),*]
            }
        }
    });

    quote! {
        #allow_attrs
        impl NodeKind {
            pub fn var_names(self) -> Vec<VarName> {
                match self {
                    #(#var_names_arms,)*
                    NodeKind::None => Vec::new(),
                }
            }
        }
    }
}

pub fn generate_collect_owned_ids_impl(node: &NodeInfo) -> proc_macro2::TokenStream {
    let collect_calls = node
        .fields
        .iter()
        .filter_map(|field| match field.link_type {
            LinkType::Owned | LinkType::OwnedMultiple | LinkType::Component => {
                let field_name = &field.name;
                Some(match field.link_type {
                    LinkType::OwnedMultiple => quote! {
                        if let Ok(many_data) = self.#field_name.get() {
                            for n in many_data {
                                v.extend(n.collect_owned_ids());
                            }
                        }
                    },
                    _ => quote! {
                        if let Ok(n) = self.#field_name.get() {
                            v.extend(n.collect_owned_ids());
                        }
                    },
                })
            }
            _ => None,
        });

    quote! {
        fn collect_owned_ids(&self) -> Vec<u64> {
            let mut v = vec![self.id];
            #(#collect_calls)*
            v
        }
    }
}

pub fn generate_collect_owned_links_impl(node: &NodeInfo) -> proc_macro2::TokenStream {
    let link_calls = node
        .fields
        .iter()
        .filter_map(|field| match field.link_type {
            LinkType::Owned | LinkType::OwnedMultiple | LinkType::Component => {
                let field_name = &field.name;
                Some(match field.link_type {
                    LinkType::OwnedMultiple => quote! {
                        if let Ok(children_data) = self.#field_name.get() {
                            for n in children_data {
                                v.push((self.id, n.id));
                                v.extend(n.collect_owned_links());
                            }
                        }
                    },
                    _ => quote! {
                        if let Ok(n) = self.#field_name.get() {
                            v.push((self.id, n.id));
                            v.extend(n.collect_owned_links());
                        }
                    },
                })
            }
            _ => None,
        });

    quote! {
        fn collect_owned_links(&self) -> Vec<(u64, u64)> {
            let mut v = Vec::new();
            #(#link_calls)*
            v
        }
    }
}

pub fn generate_manual_serialize_impl(node: &NodeInfo) -> proc_macro2::TokenStream {
    let struct_name = &node.name;

    // Get only data fields (non-link fields or fields marked with #[var])
    let data_fields: Vec<_> = node
        .fields
        .iter()
        .filter(|field| field.link_type == LinkType::None || field.is_var)
        .collect();

    if data_fields.is_empty() {
        // No data fields to serialize - serialize as unit
        return quote! {
            impl serde::Serialize for #struct_name {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer,
                {
                    serializer.serialize_unit()
                }
            }

            impl<'de> serde::Deserialize<'de> for #struct_name {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: serde::Deserializer<'de>,
                {
                    use serde::de::{self, Visitor};

                    struct UnitVisitor;
                    impl<'de> Visitor<'de> for UnitVisitor {
                        type Value = ();

                        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                            formatter.write_str("unit")
                        }

                        fn visit_unit<E>(self) -> Result<Self::Value, E>
                        where
                            E: de::Error,
                        {
                            Ok(())
                        }
                    }

                    deserializer.deserialize_unit(UnitVisitor)?;
                    Ok(Self::default())
                }
            }
        };
    }

    let serialize_values = data_fields.iter().map(|field| {
        let field_name = &field.name;
        quote! { &self.#field_name }
    });

    // Generate all other fields with default values for deserialization
    let other_fields = node.fields.iter().filter_map(|field| {
        if data_fields.iter().any(|df| df.name == field.name) {
            None // Skip data fields
        } else {
            let field_name = &field.name;
            Some(quote! { #field_name: Default::default() })
        }
    });

    let deserialize_fields = data_fields.iter().enumerate().map(|(i, field)| {
        let field_name = &field.name;
        let index = syn::Index::from(i);
        quote! { #field_name: tuple.#index }
    });

    if data_fields.len() == 1 {
        // Single value - serialize as single value, not tuple
        let field_name = &data_fields[0].name;
        quote! {
            impl serde::Serialize for #struct_name {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer,
                {
                    self.#field_name.serialize(serializer)
                }
            }

            impl<'de> serde::Deserialize<'de> for #struct_name {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: serde::Deserializer<'de>,
                {
                    let value = serde::Deserialize::deserialize(deserializer)?;
                    Ok(Self {
                        id: 0,
                        owner: 0,
                        #field_name: value,
                        #(#other_fields),*
                    })
                }
            }
        }
    } else {
        // Multiple values - serialize as tuple
        let tuple_types = data_fields.iter().map(|field| {
            let raw_type = &field.raw_type;
            let ty: syn::Type = syn::parse_str(raw_type).unwrap_or_else(|_| {
                syn::parse_quote! { String }
            });
            quote! { #ty }
        });

        quote! {
            impl serde::Serialize for #struct_name {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer,
                {
                    (#(#serialize_values),*).serialize(serializer)
                }
            }

            impl<'de> serde::Deserialize<'de> for #struct_name {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: serde::Deserializer<'de>,
                {
                    let tuple: (#(#tuple_types),*) = serde::Deserialize::deserialize(deserializer)?;
                    Ok(Self {
                        id: 0,
                        owner: 0,
                        #(#deserialize_fields),*,
                        #(#other_fields),*
                    })
                }
            }
        }
    }
}

pub fn generate_update_link_references_impl(node: &NodeInfo) -> TokenStream {
    let mut ref_update_statements = Vec::new();
    let mut owned_recursive_statements = Vec::new();

    for field in &node.fields {
        let field_name = &field.name;

        match &field.link_type {
            LinkType::Ref => {
                ref_update_statements.push(quote! {
                    match &mut self.#field_name {
                        Ref::Id(id) => {
                            if let Some(&new_id) = id_map.get(id) {
                                *id = new_id;
                            }
                        },
                        _ => {}
                    }
                });
            }
            LinkType::RefMultiple => {
                ref_update_statements.push(quote! {
                    match &mut self.#field_name {
                        RefMultiple::Ids(ids) => {
                            for id in ids.iter_mut() {
                                if let Some(&new_id) = id_map.get(id) {
                                    *id = new_id;
                                }
                            }
                        },
                        _ => {}
                    }
                });
            }
            LinkType::Owned | LinkType::Component => {
                owned_recursive_statements.push(quote! {
                    if let Ok(node) = self.#field_name.get_mut() {
                        node.reassign_ids(next_id, id_map);
                    }
                });
            }
            LinkType::OwnedMultiple => {
                owned_recursive_statements.push(quote! {
                    if let Ok(nodes) = self.#field_name.get_mut() {
                        for node in nodes.iter_mut() {
                            node.reassign_ids(next_id, id_map);
                        }
                    }
                });
            }
            LinkType::None => {}
        }
    }

    quote! {
        fn reassign_owned_ids(&mut self, next_id: &mut u64, id_map: &mut std::collections::HashMap<u64, u64>) {
            #(#owned_recursive_statements)*
        }

        fn update_reference_links(&mut self, id_map: &std::collections::HashMap<u64, u64>) {
            #(#ref_update_statements)*
        }
    }
}

pub fn generate_named_node_trait() -> proc_macro2::TokenStream {
    quote! {
        pub trait NamedNode {
            fn named_kind() -> NamedNodeKind;
            fn name(&self) -> &str;
        }
    }
}

pub fn generate_named_node_impl(node: &NodeInfo) -> proc_macro2::TokenStream {
    let struct_name = &node.name;
    let node_kind_variant = &node.name;

    let name_field = if let Some(ref field_name) = node.name_field {
        field_name.clone()
    } else {
        // Default to looking for a field that contains "name" in its name
        node.fields
            .iter()
            .find(|field| {
                let field_name = field.name.to_string();
                field_name.contains("name") && matches!(field.raw_type.as_str(), "String")
            })
            .map(|field| field.name.clone())
            .unwrap_or_else(|| {
                panic!("Named node {} must have a name field specified or a String field containing 'name'", struct_name)
            })
    };

    quote! {
        impl NamedNode for #struct_name {
            fn named_kind() -> NamedNodeKind {
                NamedNodeKind::#node_kind_variant
            }

            fn name(&self) -> &str {
                &self.#name_field
            }
        }

    }
}

pub fn generate_node_kind_match_macro(nodes: &[NodeInfo]) -> proc_macro2::TokenStream {
    let match_arms = nodes.iter().map(|node| {
        let node_kind_variant = &node.name;
        let struct_name = &node.name;

        quote! {
            NodeKind::#node_kind_variant => {
                type NodeType = #struct_name;
                $code
            }
        }
    });

    quote! {
        #[macro_export]
        macro_rules! node_kind_match {
            ($kind:expr, $code:expr) => {
                match $kind {
                    NodeKind::None => {
                        unreachable!()
                    }
                    #(#match_arms)*
                }
            };
        }
    }
}

pub fn generate_set_owner_calls(node: &NodeInfo) -> Vec<TokenStream> {
    node.fields
        .iter()
        .filter(|field| {
            matches!(
                field.link_type,
                LinkType::Owned | LinkType::OwnedMultiple | LinkType::Component
            )
        })
        .map(|field| {
            let field_name = &field.name;
            match field.link_type {
                LinkType::Owned => {
                    quote! {
                        if let schema::Owned::Loaded(_, node) = &mut self.#field_name {
                            node.set_owner(owner);
                        }
                    }
                }
                LinkType::Component => {
                    quote! {
                        if let schema::Component::Loaded(_, node) = &mut self.#field_name {
                            node.set_owner(owner);
                        }
                    }
                }
                LinkType::OwnedMultiple => {
                    quote! {
                        if let schema::OwnedMultiple::Loaded(_, nodes) = &mut self.#field_name {
                            for node in nodes.iter_mut() {
                                node.set_owner(owner);
                            }
                        }
                    }
                }
                _ => unreachable!(),
            }
        })
        .collect()
}

pub fn generate_named_node_kind_match_macro(nodes: &[NodeInfo]) -> TokenStream {
    let match_arms = nodes.iter().filter(|node| node.is_named).map(|node| {
        let node_kind_variant = &node.name;
        let struct_name = &node.name;

        quote! {
            NamedNodeKind::#node_kind_variant => {
                type NamedNodeType = #struct_name;
                $code
            }
        }
    });

    quote! {
        #[macro_export]
        macro_rules! named_node_kind_match {
            ($kind:expr, $code:expr) => {
                match $kind {
                    #(#match_arms)*
                }
            };
        }
    }
}

pub fn generate_unit_check_functions(context_type: &str) -> proc_macro2::TokenStream {
    let context_ident = format_ident!("{}", context_type);

    quote! {
        impl NUnit {
            pub fn check_fusible(&mut self, ctx: &#context_ident) -> NodeResult<bool> {
                let houses = ctx.collect_kind_parents(self.id, NodeKind::NHouse)?;

                if houses.len() == 1 {
                    // Unit with one house needs at least 2 stax to be fusible
                    let state = self.state_load(ctx)?;
                    Ok(state.stax >= 2)
                } else {
                    // Multi-house unit needs total actions >= stax to be fusible
                    let state = self.state_load(ctx)?;
                    let stax = state.stax;
                    let actions = self
                        .behavior_load(ctx)?
                        .reactions
                        .iter()
                        .map(|r| r.actions.len() as i32)
                        .sum::<i32>();
                    Ok(actions >= stax)
                }
            }

            pub fn check_stackable(&mut self, other: &mut NUnit, ctx: &#context_ident) -> NodeResult<bool> {
                let self_houses = ctx.collect_kind_parents(self.id, NodeKind::NHouse)?;
                let other_houses = ctx.collect_kind_parents(other.id, NodeKind::NHouse)?;
                if self_houses.len() == 1 && other_houses.len() == 1 {
                    return Ok(self.name() == other.name() && self_houses[0] == other_houses[0]);
                }
                Ok(self_houses.iter().all(|h| other_houses.contains(h)) ||other_houses.iter().all(|h| self_houses.contains(h)))
            }

            pub fn check_fusible_with(&mut self, other: &mut NUnit, ctx: &#context_ident) -> NodeResult<bool> {
                let self_houses = ctx.collect_kind_parents(self.id, NodeKind::NHouse)?;
                let other_houses = ctx.collect_kind_parents(other.id, NodeKind::NHouse)?;

                // Units can be fused if all of their houses are different
                for house in &self_houses {
                    if other_houses.contains(house) {
                        return Ok(false);
                    }
                }

                // Both units must also be fusible on their own
                Ok(self.check_fusible(ctx)? && other.check_fusible(ctx)?)
            }
        }
    }
}
