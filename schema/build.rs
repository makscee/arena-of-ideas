use node_build_utils::*;
use quote::{format_ident, quote};
use std::{
    collections::{HashMap, HashSet},
    env, fs,
    path::Path,
};

fn main() {
    println!("cargo:rerun-if-changed=src/raw_nodes.rs");
    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("node_kind.rs");

    let input = fs::read_to_string("src/raw_nodes.rs").expect("Failed to read raw_nodes.rs");
    let nodes = parse_nodes_file(&input).expect("Failed to parse raw_nodes.rs");

    let mut node_map = HashMap::new();
    for node in &nodes {
        node_map.insert(node.name.to_string(), node.clone());
    }

    // Validate parent relationships
    if let Err(err) = validate_parent_relationships(&node_map) {
        panic!("{}", err);
    }

    // Generate code
    let generated = generate_node_kind(&nodes, &node_map);
    let var_names_impl = generate_var_names_for_node_kind(&nodes);

    // Add comprehensive allow attributes at the top
    let allow_attrs = generated_code_allow_attrs();
    let combined = quote! {
        #allow_attrs
        #generated
        #var_names_impl
    };
    let formatted_code = format_code(&combined);
    fs::write(&dest_path, formatted_code).expect("Failed to write generated code");
}

fn generate_node_relation_arms(node_map: &HashMap<String, NodeInfo>) -> proc_macro2::TokenStream {
    use quote::quote;

    let mut arms = Vec::new();

    for (node_name, node_info) in node_map {
        for field in &node_info.fields {
            let parent_kind = node_name.parse::<proc_macro2::TokenStream>().unwrap();
            let child_kind = field
                .target_type
                .clone()
                .parse::<proc_macro2::TokenStream>()
                .unwrap();

            match field.link_type {
                LinkType::OwnedMultiple => {
                    if field.is_many_to_one {
                        arms.push(quote! {
                            (NodeKind::#parent_kind, NodeKind::#child_kind) => Some(NodeRelation::ManyToOne),
                        });
                    } else {
                        arms.push(quote! {
                            (NodeKind::#parent_kind, NodeKind::#child_kind) => Some(NodeRelation::OneToMany),
                        });
                    }
                }
                LinkType::Owned => {
                    if field.is_many_to_one {
                        arms.push(quote! {
                            (NodeKind::#parent_kind, NodeKind::#child_kind) => Some(NodeRelation::ManyToOne),
                        });
                    } else {
                        arms.push(quote! {
                            (NodeKind::#parent_kind, NodeKind::#child_kind) => Some(NodeRelation::OneToMany),
                        });
                    }
                }
                LinkType::Component => {
                    if field.is_many_to_one {
                        arms.push(quote! {
                            (NodeKind::#parent_kind, NodeKind::#child_kind) => Some(NodeRelation::ManyToOne),
                        });
                    } else {
                        arms.push(quote! {
                            (NodeKind::#parent_kind, NodeKind::#child_kind) => Some(NodeRelation::OneToOne),
                        });
                    }
                }
                LinkType::None => {
                    // Skip non-link fields
                }
            }
        }
    }

    quote! { #(#arms)* }
}

fn generate_is_one_to_many_arms(node_map: &HashMap<String, NodeInfo>) -> proc_macro2::TokenStream {
    use quote::quote;

    let mut arms = Vec::new();

    for (node_name, node_info) in node_map {
        for field in &node_info.fields {
            if matches!(field.link_type, LinkType::OwnedMultiple) && !field.is_many_to_one {
                let parent_ident = syn::parse_str::<syn::Ident>(node_name).unwrap();
                let child_ident = syn::parse_str::<syn::Ident>(&field.target_type).unwrap();
                arms.push(quote! {
                    (NodeKind::#parent_ident, NodeKind::#child_ident) => true,
                });
            }
        }
    }

    quote! {
        #(#arms)*
    }
}

fn generate_is_component_child_arms(
    component_children: &HashMap<String, HashSet<String>>,
) -> proc_macro2::TokenStream {
    let arms: Vec<_> = component_children
        .iter()
        .map(|(parent, children)| {
            let parent_ident = format_ident!("{}", parent);
            let child_patterns: Vec<_> = children
                .iter()
                .map(|child| {
                    let child_ident = format_ident!("{}", child);
                    quote! { NodeKind::#child_ident }
                })
                .collect();

            quote! {
                NodeKind::#parent_ident => {
                    matches!(self, #(#child_patterns)|*)
                }
            }
        })
        .collect();

    quote! { #(#arms)* }
}

fn generate_component_children_arms(
    component_children: &HashMap<String, HashSet<String>>,
) -> proc_macro2::TokenStream {
    let arms: Vec<_> = component_children
        .iter()
        .map(|(parent, children)| {
            let parent_ident = format_ident!("{}", parent);
            let child_idents: Vec<_> = children
                .iter()
                .map(|child| {
                    let child_ident = format_ident!("{}", child);
                    quote! { children.push(NodeKind::#child_ident); }
                })
                .collect();

            quote! {
                NodeKind::#parent_ident => {
                    #(#child_idents)*
                }
            }
        })
        .collect();

    quote! { #(#arms)* }
}

fn generate_node_kind(
    nodes: &[NodeInfo],
    node_map: &HashMap<String, NodeInfo>,
) -> proc_macro2::TokenStream {
    let node_names: Vec<_> = nodes.iter().map(|n| &n.name).collect();
    let content_nodes: Vec<_> = nodes
        .iter()
        .filter(|n| n.is_content)
        .map(|n| &n.name)
        .collect();
    let named_nodes: Vec<_> = nodes
        .iter()
        .filter(|n| n.is_named)
        .map(|n| &n.name)
        .collect();

    // Build parent-child relationships
    let relationships = build_relationship_maps(node_map);

    // Generate match arms for relationship functions
    let is_component_child_arms =
        generate_is_component_child_arms(&relationships.component_children);
    let component_children_arms =
        generate_component_children_arms(&relationships.component_children);
    let owned_parent_arms = generate_owning_parent_arms(&relationships.owned_parents);
    let owned_children_arms = generate_owning_children_arms(&relationships.owned_children);

    let allow_attrs = generated_code_allow_attrs();
    quote! {
        use std::collections::HashSet;

        #allow_attrs
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
        pub enum NodeRelation {
            OneToOne,
            OneToMany,
            ManyToOne,
        }

        #allow_attrs
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, strum_macros::Display, strum_macros::EnumIter, strum_macros::EnumString, strum_macros::AsRefStr, PartialOrd, Ord)]
        pub enum NodeKind {
            None,
            #(#node_names,)*
        }

        #allow_attrs
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, strum_macros::Display, strum_macros::EnumIter, strum_macros::EnumString, strum_macros::AsRefStr)]
        pub enum ContentNodeKind {
            #(#content_nodes,)*
        }

        #allow_attrs
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, strum_macros::Display, strum_macros::EnumIter, strum_macros::EnumString, strum_macros::AsRefStr)]
        pub enum NamedNodeKind {
            #(#named_nodes,)*
        }

        #allow_attrs
        impl Default for NodeKind {
            fn default() -> Self {
                NodeKind::None
            }
        }

        #allow_attrs
        impl NodeKind {
            pub fn is_content(self) -> bool {
                matches!(self, #(NodeKind::#content_nodes)|*)
            }

            pub fn is_named(self) -> bool {
                matches!(self, #(NodeKind::#named_nodes)|*)
            }

            pub fn is_component_child(self, parent_kind: NodeKind) -> bool {
                match parent_kind {
                    #is_component_child_arms
                    _ => false,
                }
            }

            pub fn parents(self) -> Vec<NodeKind> {
                let mut parents = Vec::new();

                // Add owned parent if exists
                match self {
                    #owned_parent_arms
                    _ => {}
                }

                parents
            }

            pub fn children(self) -> Vec<NodeKind> {
                let mut children = Vec::new();

                // Add component children
                match self {
                    #component_children_arms
                    _ => {}
                }

                // Add owned children
                match self {
                    #owned_children_arms
                    _ => {}
                }

                children
            }
        }

        #allow_attrs
        impl TryFrom<NodeKind> for ContentNodeKind {
            type Error = ();

            fn try_from(kind: NodeKind) -> Result<Self, Self::Error> {
                match kind {
                    #(NodeKind::#content_nodes => Ok(ContentNodeKind::#content_nodes),)*
                    _ => Err(()),
                }
            }
        }

        #allow_attrs
        impl From<ContentNodeKind> for NodeKind {
            fn from(content: ContentNodeKind) -> Self {
                match content {
                    #(ContentNodeKind::#content_nodes => NodeKind::#content_nodes,)*
                }
            }
        }

        #allow_attrs
        impl TryFrom<NodeKind> for NamedNodeKind {
            type Error = ();

            fn try_from(kind: NodeKind) -> Result<Self, Self::Error> {
                match kind {
                    #(NodeKind::#named_nodes => Ok(NamedNodeKind::#named_nodes),)*
                    _ => Err(()),
                }
            }
        }

        #allow_attrs
        impl From<NamedNodeKind> for NodeKind {
            fn from(named: NamedNodeKind) -> Self {
                match named {
                    #(NamedNodeKind::#named_nodes => NodeKind::#named_nodes,)*
                }
            }
        }

        #allow_attrs
        impl ToNodeKind for NamedNodeKind {
            fn to_kind(&self) -> NodeKind {
                NodeKind::from(*self)
            }
        }

        #allow_attrs
        impl ToNodeKind for ContentNodeKind {
            fn to_kind(&self) -> NodeKind {
                NodeKind::from(*self)
            }
        }
    }
}
