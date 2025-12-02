use node_build_utils::*;
use quote::quote;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;

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
                LinkType::OwnedMultiple | LinkType::RefMultiple => {
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
                LinkType::Owned | LinkType::Ref => {
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
            if matches!(
                field.link_type,
                LinkType::OwnedMultiple | LinkType::RefMultiple
            ) && !field.is_many_to_one
            {
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
    let component_parent_arms = generate_parent_arms(&relationships.component_parents);
    let component_children_arms = generate_children_arms(&relationships.component_children);
    let component_children_recursive_arms =
        generate_children_recursive_arms(&relationships.component_children);
    let owned_parent_arms = generate_owning_parent_arms(&relationships.owned_parents);
    let owned_children_arms = generate_owning_children_arms(&relationships.owned_children);
    let other_components_arms = generate_other_components_arms(
        &relationships.component_parents,
        &relationships.component_children,
    );

    // Generate is_one_to_many function
    let is_one_to_many_arms = generate_is_one_to_many_arms(node_map);

    // Generate get_relation function
    let node_relation_arms = generate_node_relation_arms(node_map);

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

            pub fn component_parent(self) -> Option<NodeKind> {
                match self {
                    #component_parent_arms
                    _ => None,
                }
            }

            pub fn component_children(self) -> HashSet<NodeKind> {
                match self {
                    #component_children_arms
                    _ => HashSet::new(),
                }
            }

            pub fn component_children_recursive(self) -> HashSet<NodeKind> {
                match self {
                    #component_children_recursive_arms
                    _ => HashSet::new(),
                }
            }

            pub fn owning_parents(self) -> Vec<NodeKind> {
                let mut parents = Vec::new();

                // Add component parent if exists
                if let Some(parent) = self.component_parent() {
                    parents.push(parent);
                }

                // Add owned parent if exists
                match self {
                    #owned_parent_arms
                    _ => {}
                }

                parents
            }

            pub fn owning_children(self) -> Vec<NodeKind> {
                let mut children = Vec::new();

                // Add component children
                children.extend(self.component_children());

                // Add owned children
                match self {
                    #owned_children_arms
                    _ => {}
                }

                children
            }

            pub fn other_components(self) -> HashSet<NodeKind> {
                match self {
                    #other_components_arms
                    _ => HashSet::new(),
                }
            }

            pub fn is_one_to_many(parent: NodeKind, child: NodeKind) -> bool {
                match (parent, child) {
                    #is_one_to_many_arms
                    _ => false,
                }
            }

            pub fn get_relation(parent: NodeKind, child: NodeKind) -> Option<NodeRelation> {
                match (parent, child) {
                    #node_relation_arms
                    _ => None,
                }
            }

            pub fn base_kind(self) -> NodeKind {
                let mut current = self;
                while let Some(parent_kind) = current.component_parent() {
                    current = parent_kind;
                }
                current
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
