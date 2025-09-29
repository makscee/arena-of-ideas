use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::{HashMap, HashSet};
use syn::*;

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
    pub is_optional: bool,
    pub is_vec: bool,
    pub raw_type: String,
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
    item_struct
        .attrs
        .iter()
        .any(|attr| attr.path().is_ident("node"))
}

pub fn parse_node(item_struct: &ItemStruct) -> NodeInfo {
    let name = item_struct.ident.clone();
    let mut is_content = false;
    let mut is_named = false;

    for attr in &item_struct.attrs {
        if attr.path().is_ident("node") {
            if let Ok(Meta::List(meta_list)) = attr.meta.clone().try_into() {
                for token in meta_list.tokens.clone() {
                    let token_str = token.to_string();
                    if token_str.contains("content") {
                        is_content = true;
                    }
                    if token_str.contains("name") {
                        is_named = true;
                    }
                }
            }
        }
    }

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
    let (link_type, target_type, is_optional, is_vec) = parse_field_type(&field.ty);

    let raw_type = match &field.ty {
        Type::Path(type_path) => quote! { #type_path }.to_string(),
        _ => String::new(),
    };

    FieldInfo {
        name,
        link_type,
        target_type,
        is_optional,
        is_vec,
        raw_type,
    }
}

pub fn parse_field_type(ty: &Type) -> (LinkType, String, bool, bool) {
    if let Type::Path(type_path) = ty {
        let path = &type_path.path;

        if let Some(segment) = path.segments.last() {
            match segment.ident.to_string().as_str() {
                "Component" => {
                    let (target, is_optional, is_vec) = parse_generic_arg(&segment.arguments);
                    return (LinkType::Component, target, is_optional, is_vec);
                }
                "Owned" => {
                    let (target, is_optional, is_vec) = parse_generic_arg(&segment.arguments);
                    return (LinkType::Owned, target, is_optional, is_vec);
                }
                "Ref" => {
                    let (target, is_optional, is_vec) = parse_generic_arg(&segment.arguments);
                    return (LinkType::Ref, target, is_optional, is_vec);
                }
                _ => {}
            }
        }
    }

    (LinkType::None, String::new(), false, false)
}

pub fn parse_generic_arg(args: &PathArguments) -> (String, bool, bool) {
    if let PathArguments::AngleBracketed(generic_args) = args {
        if let Some(GenericArgument::Type(inner_ty)) = generic_args.args.first() {
            return parse_inner_type(inner_ty);
        }
    }
    (String::new(), false, false)
}

pub fn parse_inner_type(ty: &Type) -> (String, bool, bool) {
    if let Type::Path(type_path) = ty {
        let path = &type_path.path;

        if let Some(segment) = path.segments.last() {
            match segment.ident.to_string().as_str() {
                "Option" => {
                    if let PathArguments::AngleBracketed(args) = &segment.arguments {
                        if let Some(GenericArgument::Type(inner)) = args.args.first() {
                            let (target, _, is_vec) = parse_inner_type(inner);
                            return (target, true, is_vec);
                        }
                    }
                }
                "Vec" => {
                    if let PathArguments::AngleBracketed(args) = &segment.arguments {
                        if let Some(GenericArgument::Type(inner)) = args.args.first() {
                            let (target, is_optional, _) = parse_inner_type(inner);
                            return (target, is_optional, true);
                        }
                    }
                }
                name => {
                    return (name.to_string(), false, false);
                }
            }
        }
    }
    (String::new(), false, false)
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

pub fn generate_field_type(field: &FieldInfo, schema_prefix: &str) -> TokenStream {
    match field.link_type {
        LinkType::Component => {
            let target = if field.target_type.is_empty() {
                quote! { String }
            } else {
                let target_ident = format_ident!("{}", field.target_type);
                quote! { #target_ident }
            };

            if field.is_optional {
                let component_path = if schema_prefix.is_empty() {
                    quote! { Component<Option<#target>> }
                } else {
                    let prefix = format_ident!("{}", schema_prefix);
                    quote! { #prefix::Component<Option<#target>> }
                };
                component_path
            } else {
                let component_path = if schema_prefix.is_empty() {
                    quote! { Component<#target> }
                } else {
                    let prefix = format_ident!("{}", schema_prefix);
                    quote! { #prefix::Component<#target> }
                };
                component_path
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
                let owned_path = if schema_prefix.is_empty() {
                    quote! { Owned<Vec<#target>> }
                } else {
                    let prefix = format_ident!("{}", schema_prefix);
                    quote! { #prefix::Owned<Vec<#target>> }
                };
                owned_path
            } else if field.is_optional {
                let owned_path = if schema_prefix.is_empty() {
                    quote! { Owned<Option<#target>> }
                } else {
                    let prefix = format_ident!("{}", schema_prefix);
                    quote! { #prefix::Owned<Option<#target>> }
                };
                owned_path
            } else {
                let owned_path = if schema_prefix.is_empty() {
                    quote! { Owned<#target> }
                } else {
                    let prefix = format_ident!("{}", schema_prefix);
                    quote! { #prefix::Owned<#target> }
                };
                owned_path
            }
        }
        LinkType::Ref => {
            let target = if field.target_type.is_empty() {
                quote! { String }
            } else {
                let target_ident = format_ident!("{}", field.target_type);
                quote! { #target_ident }
            };

            if field.is_optional {
                let ref_path = if schema_prefix.is_empty() {
                    quote! { Ref<Option<#target>> }
                } else {
                    let prefix = format_ident!("{}", schema_prefix);
                    quote! { #prefix::Ref<Option<#target>> }
                };
                ref_path
            } else {
                let ref_path = if schema_prefix.is_empty() {
                    quote! { Ref<#target> }
                } else {
                    let prefix = format_ident!("{}", schema_prefix);
                    quote! { #prefix::Ref<#target> }
                };
                ref_path
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

pub fn generate_accessors(node: &NodeInfo) -> Vec<TokenStream> {
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

pub fn generate_conversions(nodes: &[NodeInfo]) -> TokenStream {
    let node_trait_impls = nodes.iter().map(|node| {
        let struct_name = &node.name;
        let node_kind_variant = &node.name;

        quote! {
            impl SchemaNode for #struct_name {
                fn id(&self) -> u64 {
                    self.id.unwrap_or(0)
                }

                fn set_id(&mut self, id: u64) {
                    self.id = Some(id);
                }

                fn owner(&self) -> u64 {
                    // Client nodes don't typically track owner directly
                    0
                }

                fn set_owner(&mut self, _owner: u64) {
                    // Client nodes don't typically track owner directly
                }

                fn kind(&self) -> NodeKind {
                    NodeKind::#node_kind_variant
                }

                fn reassign_ids(&mut self, next_id: &mut u64) {
                    if self.id.is_none() {
                        self.set_id(*next_id);
                        *next_id += 1;
                    }
                }

                fn kind_s() -> NodeKind {
                    NodeKind::#node_kind_variant
                }
            }
        }
    });

    quote! {
        #(#node_trait_impls)*
    }
}
