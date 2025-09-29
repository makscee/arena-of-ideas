use node_build_utils::*;
use quote::{format_ident, quote};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=../raw-nodes-v2/src/raw_nodes.rs");
    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("client_nodes.rs");

    // Read the raw nodes file from raw-nodes-v2
    let input = fs::read_to_string("../raw-nodes-v2/src/raw_nodes.rs")
        .expect("Failed to read raw_nodes.rs");
    let nodes = parse_nodes_file(&input).expect("Failed to parse raw_nodes.rs");

    let mut node_map = HashMap::new();
    for node in &nodes {
        node_map.insert(node.name.to_string(), node.clone());
    }

    // Generate client-specific node implementations
    let generated = generate_client_nodes(&nodes, &node_map);

    // Format and write
    let formatted_code = format_code(&generated);
    fs::write(&dest_path, formatted_code).expect("Failed to write generated code");
}

fn generate_client_nodes(
    nodes: &[NodeInfo],
    _node_map: &HashMap<String, NodeInfo>,
) -> proc_macro2::TokenStream {
    let node_structs = nodes.iter().map(|node| {
        let struct_name = &node.name;

        // Generate fields
        let fields = node.fields.iter().map(|field| {
            let field_name = &field.name;
            let field_type = generate_field_type(field);

            quote! {
                pub #field_name: #field_type
            }
        });

        // Generate accessor methods
        let accessors = generate_accessors(node);

        // All nodes are Components in client
        let derives = quote! {
            #[derive(Debug, Clone, Component, Serialize, Deserialize)]
        };

        quote! {
            #derives
            pub struct #struct_name {
                pub id: Option<u64>,
                #(#fields,)*
            }

            impl #struct_name {
                pub fn new() -> Self {
                    Self {
                        id: None,
                        #(#accessors)*
                    }
                }

                pub fn with_id(mut self, id: u64) -> Self {
                    self.id = Some(id);
                    self
                }
            }

            impl Default for #struct_name {
                fn default() -> Self {
                    Self::new()
                }
            }
        }
    });

    // Generate conversion traits
    let conversions = generate_conversions(nodes);

    // Generate HasNodeKind implementations
    let has_node_kind_impls = nodes.iter().map(|node| {
        let struct_name = &node.name;
        let node_kind_variant = &node.name;

        quote! {
            impl HasNodeKind for #struct_name {
                fn node_kind() -> NodeKind {
                    NodeKind::#node_kind_variant
                }
            }
        }
    });

    quote! {
        use bevy::prelude::*;
        use serde::{Deserialize, Serialize};
        use schema_v2::{NodeKind, HasNodeKind, Node as NodeTrait};
        use raw_nodes_v2::{HexColor, Action, Reaction, Material, ShopOffer, UnitActionRange, MagicType};
        use raw_nodes_v2::Trigger as RawTrigger;

        #(#node_structs)*

        #conversions

        #(#has_node_kind_impls)*
    }
}

fn generate_field_type(field: &FieldInfo) -> proc_macro2::TokenStream {
    match field.link_type {
        LinkType::Component => {
            let target = if field.target_type.is_empty() {
                quote! { String }
            } else {
                let target_ident = format_ident!("{}", field.target_type);
                quote! { #target_ident }
            };

            if field.is_optional {
                quote! { Component<Option<#target>> }
            } else {
                quote! { schema_v2::Component<#target> }
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
                quote! { schema_v2::Owned<Vec<#target>> }
            } else if field.is_optional {
                quote! { schema_v2::Owned<Option<#target>> }
            } else {
                quote! { schema_v2::Owned<#target> }
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
                quote! { schema_v2::Ref<Option<#target>> }
            } else {
                quote! { schema_v2::Ref<#target> }
            }
        }
        LinkType::None => {
            // For primitive types, use the raw type directly
            if field.raw_type.is_empty() {
                quote! { String }
            } else {
                // Replace Trigger with RawTrigger to avoid conflict with bevy::prelude::Trigger
                let raw_type = field.raw_type.replace("Trigger", "RawTrigger");
                let tokens: proc_macro2::TokenStream =
                    raw_type.parse().unwrap_or_else(|_| quote! { String });
                tokens
            }
        }
    }
}

fn generate_accessors(node: &NodeInfo) -> Vec<proc_macro2::TokenStream> {
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

fn generate_conversions(nodes: &[NodeInfo]) -> proc_macro2::TokenStream {
    let node_trait_impls = nodes.iter().map(|node| {
        let struct_name = &node.name;
        let node_kind_variant = &node.name;

        quote! {
            impl NodeTrait for #struct_name {
                fn id(&self) -> Option<u64> {
                    self.id
                }

                fn kind(&self) -> NodeKind {
                    NodeKind::#node_kind_variant
                }
            }
        }
    });

    quote! {
        #(#node_trait_impls)*
    }
}
