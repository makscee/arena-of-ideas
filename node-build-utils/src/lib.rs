use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::{HashMap, HashSet};
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
    pub fields: Vec<FieldInfo>,
}

#[derive(Debug, Clone)]
pub struct FieldInfo {
    pub name: Ident,
    pub link_type: LinkType,
    pub target_type: String,
    pub is_vec: bool,
    pub raw_type: String,
    pub is_var: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LinkType {
    Component,
    Owned,
    Ref,
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

pub fn parse_node(item_struct: &ItemStruct) -> NodeInfo {
    let name = item_struct.ident.clone();
    let is_content = has_content_attribute(item_struct);
    let is_named = has_named_attribute(item_struct);

    let fields = item_struct
        .fields
        .iter()
        .map(|field| parse_field(field))
        .collect();

    NodeInfo {
        name,
        is_content,
        is_named,
        fields,
    }
}

pub fn parse_field(field: &Field) -> FieldInfo {
    let name = field.ident.clone().unwrap();
    let (link_type, target_type, is_vec) = parse_field_type(&field.ty);

    let raw_type = match &field.ty {
        Type::Path(type_path) => quote! { #type_path }.to_string(),
        _ => String::new(),
    };

    let is_var = field.attrs.iter().any(|attr| attr.path().is_ident("var"));

    FieldInfo {
        name,
        link_type,
        target_type,
        is_vec,
        raw_type,
        is_var,
    }
}

pub fn parse_field_type(ty: &Type) -> (LinkType, String, bool) {
    if let Type::Path(type_path) = ty {
        let path = &type_path.path;

        if let Some(segment) = path.segments.last() {
            match segment.ident.to_string().as_str() {
                "Component" => {
                    let (target, is_vec) = parse_generic_arg(&segment.arguments);
                    return (LinkType::Component, target, is_vec);
                }
                "Owned" => {
                    let (target, is_vec) = parse_generic_arg(&segment.arguments);
                    return (LinkType::Owned, target, is_vec);
                }
                "Ref" => {
                    let (target, is_vec) = parse_generic_arg(&segment.arguments);
                    return (LinkType::Ref, target, is_vec);
                }
                _ => {}
            }
        }
    }

    (LinkType::None, String::new(), false)
}

pub fn parse_generic_arg(arguments: &PathArguments) -> (String, bool) {
    if let PathArguments::AngleBracketed(args) = arguments {
        if let Some(GenericArgument::Type(ty)) = args.args.first() {
            return parse_inner_type(ty);
        }
    }
    (String::new(), false)
}

pub fn parse_inner_type(ty: &Type) -> (String, bool) {
    if let Type::Path(type_path) = ty {
        let path = &type_path.path;

        if let Some(segment) = path.segments.last() {
            match segment.ident.to_string().as_str() {
                "Vec" => {
                    if let PathArguments::AngleBracketed(args) = &segment.arguments {
                        if let Some(GenericArgument::Type(inner)) = args.args.first() {
                            let (target, _) = parse_inner_type(inner);
                            return (target, true);
                        }
                    }
                }
                name => {
                    return (name.to_string(), false);
                }
            }
        }
    }
    (String::new(), false)
}

pub fn validate_parent_relationships(
    node_map: &HashMap<String, NodeInfo>,
) -> std::result::Result<(), String> {
    // Check for Vec<Component> which is not allowed
    for (node_name, node_info) in node_map {
        for field in &node_info.fields {
            if field.is_vec && field.link_type == LinkType::Component {
                return Err(format!(
                    "Node {} has Vec<Component<{}>> which is not allowed. Components can't be Vec.",
                    node_name, field.target_type
                ));
            }
        }
    }

    // Only validate content nodes for single content parent restriction
    let mut content_parent_map: HashMap<String, Vec<String>> = HashMap::new();

    for (node_name, node_info) in node_map {
        if !node_info.is_content {
            continue;
        }

        for field in &node_info.fields {
            if field.link_type == LinkType::Component || field.link_type == LinkType::Owned {
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
                LinkType::Owned => {
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

            if field.is_vec {
                quote! { Component<Vec<#target>> }
            } else {
                quote! { Component<#target> }
            }
        }
        LinkType::Owned => {
            let target = if field.target_type.is_empty() {
                quote! { String }
            } else {
                let target_ident = format_ident!("{}", field.target_type);
                quote! { #target_ident }
            };

            if field.is_vec {
                quote! { Owned<Vec<#target>> }
            } else {
                quote! { Owned<#target> }
            }
        }
        LinkType::Ref => {
            let target = if field.target_type.is_empty() {
                quote! { String }
            } else {
                let target_ident = format_ident!("{}", field.target_type);
                quote! { #target_ident }
            };

            if field.is_vec {
                quote! { Ref<Vec<#target>> }
            } else {
                quote! { Ref<#target> }
            }
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

pub fn generate_default_accessors(node: &NodeInfo) -> Vec<TokenStream> {
    node.fields
        .iter()
        .map(|field| {
            let field_name = &field.name;
            match field.link_type {
                LinkType::Component | LinkType::Owned | LinkType::Ref => {
                    quote! { #field_name: Default::default(), }
                }
                LinkType::None => {
                    // Generate default values for primitive types
                    if field.raw_type.contains("Option") {
                        quote! { #field_name: None, }
                    } else if field.raw_type.contains("String") {
                        quote! { #field_name: String::new(), }
                    } else if field.raw_type.contains("i32") {
                        quote! { #field_name: 0, }
                    } else if field.raw_type.contains("u64") {
                        quote! { #field_name: 0, }
                    } else if field.raw_type.contains("bool") {
                        quote! { #field_name: false, }
                    } else if field.raw_type.contains("Vec") {
                        quote! { #field_name: Vec::new(), }
                    } else {
                        quote! { #field_name: Default::default(), }
                    }
                }
            }
        })
        .collect()
}

pub fn generate_new(node: &NodeInfo) -> TokenStream {
    let data_fields: Vec<_> = node
        .fields
        .iter()
        .filter(|field| field.link_type == LinkType::None)
        .collect();

    let params = std::iter::once(quote! { owner_id: u64 }).chain(data_fields.iter().map(|field| {
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
            quote! { #field_name: Default::default(), }
        });

    quote! {
        pub fn new(#(#params),*) -> Self {
            Self {
                id: 0,
                owner: owner_id,
                #(#field_assignments)*
                #(#component_defaults)*
            }
        }
    }
}

pub fn generate_add_components(node: &NodeInfo) -> TokenStream {
    let component_fields: Vec<_> = node
        .fields
        .iter()
        .filter(|field| {
            matches!(
                field.link_type,
                LinkType::Component | LinkType::Owned | LinkType::Ref
            )
        })
        .collect();

    if component_fields.is_empty() {
        return quote! {};
    }

    let params = component_fields.iter().map(|field| {
        let field_name = &field.name;
        let target = if field.target_type.is_empty() {
            quote! { String }
        } else {
            let target_ident = format_ident!("{}", field.target_type);
            quote! { #target_ident }
        };

        let param_type = if field.is_vec {
            quote! { Vec<#target> }
        } else {
            quote! { #target }
        };

        quote! { #field_name: #param_type }
    });

    let field_assignments = component_fields.iter().map(|field| {
        let field_name = &field.name;
        let wrapped_value = match field.link_type {
            LinkType::Component => {
                quote! { Component::new_loaded(#field_name) }
            }
            LinkType::Owned => {
                quote! { Owned::new_loaded(#field_name) }
            }
            LinkType::Ref => {
                quote! { Ref::new_loaded(#field_name) }
            }
            _ => quote! { #field_name },
        };
        quote! { self.#field_name = #wrapped_value; }
    });

    quote! {
        pub fn add_components(mut self, #(#params),*) -> Self {
            #(#field_assignments)*
            self
        }
    }
}

pub fn generate_default_impl(node: &NodeInfo) -> TokenStream {
    let struct_name = &node.name;
    let data_fields: Vec<_> = node
        .fields
        .iter()
        .filter(|field| field.link_type == LinkType::None)
        .collect();

    let default_params = std::iter::once(quote! { 0 }).chain(data_fields.iter().map(|field| {
        if field.raw_type.contains("Option") {
            quote! { None }
        } else if field.raw_type.contains("String") {
            quote! { String::new() }
        } else if field.raw_type.contains("i32") {
            quote! { 0 }
        } else if field.raw_type.contains("u64") {
            quote! { 0 }
        } else if field.raw_type.contains("bool") {
            quote! { false }
        } else if field.raw_type.contains("Vec") {
            quote! { Vec::new() }
        } else {
            quote! { Default::default() }
        }
    }));

    quote! {
        impl Default for #struct_name {
            fn default() -> Self {
                Self::new(#(#default_params),*)
            }
        }
    }
}

pub fn generate_node_impl(nodes: &[NodeInfo]) -> TokenStream {
    let node_trait_impls = nodes.iter().map(|node| {
        let struct_name = &node.name;
        let node_kind_variant = &node.name;
        let pack_links_impl = generate_pack_links_impl(node);
        let unpack_links_impl = generate_unpack_links_impl(node);
        let var_methods = generate_var_methods(node);
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
                }

                fn kind(&self) -> NodeKind {
                    NodeKind::#node_kind_variant
                }

                fn reassign_ids(&mut self, next_id: &mut u64) {
                    if self.id == 0 {
                        self.set_id(*next_id);
                        *next_id += 1;
                    }
                }

                fn kind_s() -> NodeKind {
                    NodeKind::#node_kind_variant
                }

                #var_methods

                #pack_links_impl
                #unpack_links_impl
            }
        }
    });

    quote! {
        #(#node_trait_impls)*
    }
}

/// Generate link loading methods for server nodes
pub fn generate_server_link_methods(node: &NodeInfo) -> TokenStream {
    let link_methods = node.fields.iter().filter_map(|field| {
        if matches!(field.link_type, LinkType::None) {
            return None;
        }
        let field_name = &field.name;
        let target_type = format_ident!("{}", field.target_type);

        let load_method = format_ident!("{}_load", field_name);
        let load_id_method = format_ident!("{}_load_id", field_name);

        if field.is_vec {
            Some(quote! {
                pub fn #load_id_method(&mut self, ctx: &ServerContext) -> Result<Vec<u64>, NodeError> {
                    if let Some(ids) = self.#field_name.ids() {
                        return Ok(ids);
                    }

                    let children = ctx.get_children_of_kind(self.id, NodeKind::#target_type)?;
                    if !children.is_empty() {
                        *self.#field_name.state_mut() = LinkState::Ids(children);
                        Ok(self.#field_name.ids().unwrap())
                    } else {
                        *self.#field_name.state_mut() = LinkState::None;
                        Ok(Vec::new())
                    }
                }

                pub fn #load_method(&mut self, ctx: &ServerContext) -> Result<&mut Vec<#target_type>, NodeError>
                {
                    if self.#field_name.is_loaded() {
                        return Ok(self.#field_name.get_mut().unwrap());
                    }
                    let ids = self.#load_id_method(ctx)?;
                    let loaded_nodes = ids
                        .iter()
                        .filter_map(|&id| ctx.load::<#target_type>(id).ok())
                        .collect_vec();
                    *self.#field_name.state_mut() = LinkState::Loaded(loaded_nodes);
                    Ok(self.#field_name.get_mut().unwrap())
                }
            })
        } else {
            Some(quote! {
                pub fn #load_id_method(&mut self, ctx: &ServerContext) -> Result<u64, NodeError> {
                    if !self.#field_name.is_none() && self.#field_name.id().is_some() {
                        return Ok(self.#field_name.id().unwrap());
                    }
                    let children = ctx.get_children_of_kind(self.id, NodeKind::#target_type)?;
                    if let Some(&first_id) = children.first() {
                        *self.#field_name.state_mut() = LinkState::Id(first_id);
                        Ok(first_id)
                    } else {
                        *self.#field_name.state_mut() = LinkState::None;
                        Err(NodeError::NotFound(self.id()))
                    }
                }

                pub fn #load_method(&mut self, ctx: &ServerContext) -> Result<&mut #target_type, NodeError>
                {
                    if self.#field_name.is_loaded() {
                        return Ok(self.#field_name.get_mut().unwrap());
                    }
                    let id = self.#load_id_method(ctx)?;
                    let loaded_node = ctx.load::<#target_type>(id)?;
                    *self.#field_name.state_mut() = LinkState::Loaded(loaded_node);
                    Ok(self.#field_name.get_mut().unwrap())
                }
            })
        }
    }).collect::<Vec<_>>();

    quote! {
        #(#link_methods)*
    }
}

pub fn generate_client_link_methods(node: &NodeInfo) -> TokenStream {
    let link_methods = node.fields.iter().filter_map(|field| {
        if matches!(field.link_type, LinkType::None) {
            return None;
        }
        let field_name = &field.name;
        let target_type = format_ident!("{}", field.target_type);

        let load_method = format_ident!("{}_load", field_name);
        let load_id_method = format_ident!("{}_load_id", field_name);
        let ref_method = format_ident!("{}_ref", field_name);

        if field.is_vec {
            Some(quote! {
                pub fn #load_id_method(&mut self, ctx: &ClientContext) -> Result<Vec<u64>, NodeError> {
                    if let Some(ids) = self.#field_name.ids() {
                        return Ok(ids.clone());
                    }

                    let children = ctx.get_children_of_kind(self.id, NodeKind::#target_type)?;
                    if !children.is_empty() {
                        *self.#field_name.state_mut() = LinkState::Ids(children.clone());
                        Ok(children)
                    } else {
                        *self.#field_name.state_mut() = LinkState::None;
                        Ok(Vec::new())
                    }
                }

                pub fn #load_method(&mut self, ctx: &ClientContext) -> Result<&mut Vec<#target_type>, NodeError>
                {
                    if self.#field_name.is_loaded() {
                        return Ok(self.#field_name.get_mut().unwrap());
                    }
                    let ids = self.#load_id_method(ctx)?;
                    let loaded_nodes = ids
                        .iter()
                        .filter_map(|&id| ctx.load::<#target_type>(id).cloned().ok())
                        .collect_vec();
                    *self.#field_name.state_mut() = LinkState::Loaded(loaded_nodes);
                    Ok(self.#field_name.get_mut().unwrap())
                }

                pub fn #ref_method<'a>(&'a self, ctx: &'a ClientContext) -> Result<Vec<&'a #target_type>, NodeError>
                {
                    let ids = if let Some(ids) = self.#field_name.ids() {
                        ids.clone()
                    } else if let Ok(ids) = ctx.get_children_of_kind(self.id, NodeKind::#target_type) {
                        ids
                    } else {
                        return Ok(Vec::new());
                    };
                    ctx.load_many::<#target_type>(&ids)
                }
            })
        } else {
            Some(quote! {
                pub fn #load_id_method(&mut self, ctx: &ClientContext) -> Result<u64, NodeError> {
                    if !self.#field_name.is_none() && self.#field_name.id().is_some() {
                        return Ok(self.#field_name.id().unwrap());
                    }
                    let children = ctx.get_children_of_kind(self.id, NodeKind::#target_type)?;
                    if let Some(&first_id) = children.first() {
                        *self.#field_name.state_mut() = LinkState::Id(first_id);
                        Ok(first_id)
                    } else {
                        *self.#field_name.state_mut() = LinkState::None;
                        Err(NodeError::NotFound(self.id()))
                    }
                }

                pub fn #load_method<'a>(&'a mut self, ctx: &'a ClientContext) -> Result<&'a mut #target_type, NodeError>
                {
                    if self.#field_name.is_loaded() {
                        return Ok(self.#field_name.get_mut().unwrap());
                    }
                    let id = self.#load_id_method(ctx)?;
                    let loaded_node = ctx.load::<#target_type>(id).cloned()?;
                    *self.#field_name.state_mut() = LinkState::Loaded(loaded_node);
                    Ok(self.#field_name.get_mut().unwrap())
                }

                pub fn #ref_method<'a>(&'a self, ctx: &'a ClientContext) -> Result<&'a #target_type, NodeError>
                {
                    if self.#field_name.is_loaded() {
                        return Ok(self.#field_name.get().unwrap());
                    }
                    let id = if let Some(id) = self.#field_name.id() {
                        id
                    } else if let Some(id) = ctx.get_children_of_kind(self.id, NodeKind::#target_type)?
                        .into_iter()
                        .next() {
                        id
                    } else {
                        return Err(NodeError::NotFound(self.id()));
                    };
                    ctx.load::<#target_type>(id)
                }
            })
        }
    }).collect::<Vec<_>>();

    quote! {
        #(#link_methods)*
    }
}

pub fn generate_load_functions(node: &NodeInfo, context_ident: &str) -> TokenStream {
    let context_ident = Ident::new(context_ident, proc_macro2::Span::call_site());
    let component_fields: Vec<_> = node
        .fields
        .iter()
        .filter(|field| field.link_type == LinkType::Component)
        .collect();

    let owned_fields: Vec<_> = node
        .fields
        .iter()
        .filter(|field| field.link_type == LinkType::Owned)
        .collect();

    // Generate load_components method (only Component links, recursive)
    let component_field_loads = component_fields.iter().map(|field| {
        let field_name = &field.name;
        let load_method = format_ident!("{}_load", field_name);

        if field.is_vec {
            quote! {
                if let Ok(loaded_items) = self.#load_method(ctx) {
                    for item in loaded_items.iter_mut() {
                        item.load_components(ctx)?;
                    }
                }
            }
        } else {
            quote! {
                if let Ok(loaded_item) = self.#load_method(ctx) {
                    loaded_item.load_components(ctx)?;
                }
            }
        }
    });

    let load_components_method = if component_fields.is_empty() {
        quote! {
            pub fn load_components(&mut self, _ctx: &#context_ident) -> Result<&mut Self, NodeError> {
                Ok(self)
            }
        }
    } else {
        quote! {
            pub fn load_components(&mut self, ctx: &#context_ident) -> Result<&mut Self, NodeError> {
                #(#component_field_loads)*
                Ok(self)
            }
        }
    };

    // Generate load_all method (Component + Owned links, recursive)
    let all_field_loads = node.fields.iter().filter_map(|field| {
        if matches!(field.link_type, LinkType::Component | LinkType::Owned) {
            let field_name = &field.name;
            let load_method = format_ident!("{}_load", field_name);

            Some(if field.is_vec {
                quote! {
                    if let Ok(loaded_items) = self.#load_method(ctx) {
                        for item in loaded_items.iter_mut() {
                            item.load_all(ctx)?;
                        }
                    }
                }
            } else {
                quote! {
                    if let Ok(loaded_item) = self.#load_method(ctx) {
                        loaded_item.load_all(ctx)?;
                    }
                }
            })
        } else {
            None
        }
    });

    let load_all_method = if component_fields.is_empty() && owned_fields.is_empty() {
        quote! {
            pub fn load_all(&mut self, _ctx: &#context_ident) -> Result<&mut Self, NodeError> {
                Ok(self)
            }
        }
    } else {
        quote! {
            pub fn load_all(&mut self, ctx: &#context_ident) -> Result<&mut Self, NodeError> {
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

        Some(if field.is_vec {
            match field.link_type {
                LinkType::Owned | LinkType::Component => quote! {
                    for item in &self.#field_name {
                        item.pack_recursive(packed, visited);
                        packed.link_parent_child(
                            self.id,
                            item.id(),
                            stringify!(#struct_name).to_string(),
                            stringify!(#target_type).to_string()
                        );
                    }
                },
                LinkType::Ref => quote! {
                    for item in &self.#field_name {
                        item.pack_recursive(packed, visited);
                        packed.link_parent_child(
                            self.id,
                            item.id(),
                            stringify!(#struct_name).to_string(),
                            stringify!(#target_type).to_string()
                        );
                    }
                },
                _ => quote! {},
            }
        } else {
            match field.link_type {
                LinkType::Component | LinkType::Owned => quote! {
                    if let Some(loaded) = self.#field_name.get() {
                        loaded.pack_recursive(packed, visited);
                        packed.link_parent_child(
                            self.id,
                            loaded.id(),
                            stringify!(#struct_name).to_string(),
                            stringify!(#target_type).to_string()
                        );
                    }
                },
                LinkType::Ref => quote! {
                    if let Some(loaded) = self.#field_name.get() {
                        loaded.pack_recursive(packed, visited);
                        packed.link_parent_child(
                            self.id,
                            loaded.id(),
                            stringify!(#struct_name).to_string(),
                            stringify!(#target_type).to_string()
                        );
                    }
                },
                _ => quote! {},
            }
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

        Some(if field.is_vec {
            match field.link_type {
                LinkType::Owned => quote! {
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
                        self.#field_name = Owned::new_loaded(children);
                    }
                },
                LinkType::Component => quote! {
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
                        self.#field_name = Component::new_loaded(children);
                    }
                },
                LinkType::Ref => quote! {
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
                        self.#field_name = Ref::new_loaded(children);
                    }
                },
                _ => quote! {},
            }
        } else {
            match field.link_type {
                LinkType::Component => quote! {
                    let child_ids = packed.kind_children(self.id, stringify!(#target_type));
                    if let Some(&child_id) = child_ids.first() {
                        if let Some(child_data) = packed.get(child_id) {
                            let mut child = #target_type::default();
                            child.inject_data(&child_data.data).unwrap();
                            child.set_id(child_id);
                            child.unpack_links(packed);
                            self.#field_name = Component::new_loaded(child);
                        }
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
                            self.#field_name = Owned::new_loaded(child);
                        }
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
                            self.#field_name = Ref::new_loaded(child);
                        }
                    }
                },
                _ => quote! {},
            }
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
        fn var_names() -> std::collections::HashSet<VarName>
        where
            Self: Sized,
        {
            let mut set = std::collections::HashSet::new();
            #(set.insert(#var_names);)*
            set
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
                _ => Err(NodeError::Custom(format!("Variable {:?} not found", var))),
            }
        }
    };

    // Generate get_vars method
    let get_vars_inserts: Vec<_> = var_fields
        .iter()
        .map(|f| {
            let field_name = &f.name;
            quote! {
                map.insert(VarName::#field_name, self.#field_name.clone().into());
            }
        })
        .collect();

    let get_vars_impl = quote! {
        fn get_vars(&self) -> std::collections::HashMap<VarName, VarValue> {
            let mut map = std::collections::HashMap::new();
            #(#get_vars_inserts)*
            map
        }
    };

    // Generate set_var method
    let set_var_arms: Vec<_> = var_fields
        .iter()
        .map(|f| {
            let field_name = &f.name;
            quote! {
                VarName::#field_name => {
                    self.#field_name = value.try_into().map_err(|_| NodeError::Custom("Value conversion failed".into()))?;
                    Ok(())
                }
            }
        })
        .collect();

    let set_var_impl = quote! {
        fn set_var(&mut self, var: VarName, value: VarValue) -> NodeResult<()> {
            match var {
                #(#set_var_arms,)*
                _ => Err(NodeError::Custom(format!("Variable {:?} not found", var))),
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
                #allow_attrs
                let mut set = std::collections::HashSet::new();
                #(set.insert(#var_names);)*
                set
            }
        }
    });

    quote! {
        #allow_attrs
        impl NodeKind {
            pub fn var_names(self) -> std::collections::HashSet<VarName> {
                match self {
                    #(#var_names_arms,)*
                    NodeKind::None => std::collections::HashSet::new(),
                }
            }

            pub fn get_var<S: ContextSource>(self, ctx: &Context<S>, node_id: u64, var: VarName) -> NodeResult<VarValue> {
                ctx.source().load_and_get_var(self, node_id, var)
            }

            pub fn set_var<S: ContextSource>(self, ctx: &mut Context<S>, node_id: u64, var: VarName, value: VarValue) -> NodeResult<()> {
                ctx.source_mut().load_and_set_var(self, node_id, var, value)
            }

            pub fn get_vars<S: ContextSource>(self, ctx: &Context<S>, node_id: u64) -> std::collections::HashMap<VarName, VarValue> {
                let mut vars = std::collections::HashMap::new();
                for var_name in self.var_names() {
                    if let Ok(value) = self.get_var(ctx, node_id, var_name) {
                        vars.insert(var_name, value);
                    }
                }
                vars
            }
        }
    }
}

pub fn generate_collect_owned_ids_impl(node: &NodeInfo) -> proc_macro2::TokenStream {
    let collect_calls = node
        .fields
        .iter()
        .filter_map(|field| match field.link_type {
            LinkType::Owned | LinkType::Component => {
                let field_name = &field.name;
                Some(if field.is_vec {
                    quote! {
                        if let Some(many_data) = self.#field_name.get() {
                            for n in many_data {
                                v.extend(n.collect_owned_ids());
                            }
                        }
                    }
                } else {
                    quote! {
                        if let Some(n) = self.#field_name.get() {
                            v.extend(n.collect_owned_ids());
                        }
                    }
                })
            }
            _ => None,
        });

    quote! {
        pub fn collect_owned_ids(&self) -> Vec<u64> {
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
            LinkType::Owned | LinkType::Component => {
                let field_name = &field.name;
                Some(if field.is_vec {
                    quote! {
                        if let Some(children_data) = self.#field_name.get() {
                            for n in children_data {
                                v.push((self.id, n.id));
                                v.extend(n.collect_owned_links());
                            }
                        }
                    }
                } else {
                    quote! {
                        if let Some(n) = self.#field_name.get() {
                            v.push((self.id, n.id));
                            v.extend(n.collect_owned_links());
                        }
                    }
                })
            }
            _ => None,
        });

    quote! {
        pub fn collect_owned_links(&self) -> Vec<(u64, u64)> {
            let mut v = Vec::new();
            #(#link_calls)*
            v
        }
    }
}
