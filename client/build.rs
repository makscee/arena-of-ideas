use node_build_utils::*;
use quote::ToTokens;
use quote::format_ident;
use quote::quote;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=../schema/src/raw_nodes.rs");
    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("client_nodes.rs");

    // Read the raw nodes file from schema
    let input =
        fs::read_to_string("../schema/src/raw_nodes.rs").expect("Failed to read raw_nodes.rs");
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
    node_map: &HashMap<String, NodeInfo>,
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

        // Generate new() method with parameters
        let new_method = generate_new(node);

        // Generate with_components() method
        let with_components_method = generate_with_components(node);

        // Generate default implementation
        let default_impl = generate_default_impl(node);

        // Generate ClientNode implementation
        let client_node_impl = generate_client_node_impl(node, node_map);

        // Generate link loading methods
        let link_methods = generate_link_methods(
            node,
            format_ident!("ClientContext"),
            Some(quote! {.cloned()}.to_token_stream()),
        );

        // All nodes are Components in client
        let derives = quote! {
            #[derive(Debug, Clone, BevyComponent, Serialize, Deserialize)]
        };

        quote! {
            #derives
            pub struct #struct_name {
                pub id: u64,
                pub owner: u64,
                #(#fields,)*
            }

            impl #struct_name {
                #new_method

                #with_components_method

                pub fn with_id(mut self, id: u64) -> Self {
                    self.id = id;
                    self
                }

                #link_methods
            }

            #client_node_impl

            #default_impl
        }
    });

    // Generate conversion traits
    let conversions = generate_conversions(nodes);

    // Generate ToCstr and FDisplay implementations
    let tocstr_impls = nodes.iter().map(|node| {
        let struct_name = &node.name;
        quote! {
            impl ToCstr for #struct_name {
                fn cstr(&self) -> Cstr {
                    format!("{}({})", stringify!(#struct_name), self.id)
                }
            }
        }
    });

    // Generate NamedNode trait and implementations
    let named_node_trait = quote! {
        pub trait NamedNode {
            fn named_kind() -> NamedNodeKind;
        }
    };

    let named_node_impls = nodes.iter().filter(|node| node.is_named).map(|node| {
        let struct_name = &node.name;
        let node_kind_variant = &node.name;

        quote! {
            impl NamedNode for #struct_name {
                fn named_kind() -> NamedNodeKind {
                    NamedNodeKind::#node_kind_variant
                }
            }
        }
    });

    quote! {
        use schema::{Context, ContextSource, NodeError, NodeKind, LinkState, Link};
        use crate::resources::context::ClientContextExt;

        #(#node_structs)*

        #conversions

        #(#tocstr_impls)*

        #named_node_trait

        #(#named_node_impls)*
    }
}

fn generate_client_node_impl(
    node: &NodeInfo,
    _node_map: &HashMap<String, NodeInfo>,
) -> proc_macro2::TokenStream {
    let struct_name = &node.name;

    // Generate spawn implementation that handles entity creation and linking
    let spawn_components = node
        .fields
        .iter()
        .filter_map(|field| match field.link_type {
            LinkType::Component => {
                let field_name = &field.name;
                Some(if field.is_optional {
                    if field.is_vec {
                        quote! {
                            for item in &self.#field_name {
                                if let Some(component) = item.as_ref() {
                                    if let Some(loaded) = component.get() {
                                        world.entity_mut(entity).insert(loaded.clone());
                                    }
                                }
                            }
                        }
                    } else {
                        quote! {
                            if let Some(component) = &self.#field_name {
                                if let Some(loaded) = component.get() {
                                    world.entity_mut(entity).insert(loaded.clone());
                                }
                            }
                        }
                    }
                } else {
                    if field.is_vec {
                        quote! {
                            for component in &self.#field_name {
                                world.entity_mut(entity).insert(component.clone());
                            }
                        }
                    } else {
                        quote! {
                            if let Some(loaded) = self.#field_name.get() {
                                world.entity_mut(entity).insert(loaded.clone());
                            }
                        }
                    }
                })
            }
            _ => None,
        });

    let spawn_owned = node
        .fields
        .iter()
        .filter_map(|field| match field.link_type {
            LinkType::Owned => {
                let field_name = &field.name;
                Some(if field.is_optional {
                    if field.is_vec {
                        quote! {
                            for item in &self.#field_name {
                                if let Some(owned) = item.as_ref() {
                                    if let Some(loaded) = owned.get() {
                                        let child_entity = world.spawn_empty().id();
                                        loaded.clone().spawn(world);
                                        world.entity_mut(child_entity).set_parent(entity);
                                    }
                                }
                            }
                        }
                    } else {
                        quote! {
                            if let Some(owned) = &self.#field_name {
                                if let Some(loaded) = owned.get() {
                                    let child_entity = world.spawn_empty().id();
                                    loaded.clone().spawn(world);
                                    world.entity_mut(child_entity).set_parent(entity);
                                }
                            }
                        }
                    }
                } else {
                    if field.is_vec {
                        quote! {
                            for owned in &self.#field_name {
                                let child_entity = world.spawn_empty().id();
                                owned.clone().spawn(world);
                                world.entity_mut(child_entity).set_parent(entity);
                            }
                        }
                    } else {
                        quote! {
                            if let Some(loaded) = self.#field_name.get() {
                                let child_entity = world.spawn_empty().id();
                                loaded.clone().spawn(world);
                                world.entity_mut(child_entity).set_parent(entity);
                            }
                        }
                    }
                })
            }
            _ => None,
        });

    let spawn_refs = node
        .fields
        .iter()
        .filter_map(|field| match field.link_type {
            LinkType::Ref => {
                let field_name = &field.name;
                Some(if field.is_optional {
                    if field.is_vec {
                        quote! {
                            for item in &self.#field_name {
                                if let Some(ref_link) = item.as_ref() {
                                    if let Some(id) = ref_link.id() {
                                        // Check if entity with this id already exists
                                        let mut found_entity = None;
                                        if let Some(node_entity_map) = world.get_resource::<NodeEntityMap>() {
                                            found_entity = node_entity_map.get_entity(id);
                                        }
                                        if found_entity.is_none() {
                                            if let Some(loaded) = ref_link.get() {
                                                loaded.clone().spawn(world);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        quote! {
                            if let Some(ref_link) = &self.#field_name {
                                if let Some(id) = ref_link.id() {
                                    // Check if entity with this id already exists
                                    let mut found_entity = None;
                                    if let Some(node_entity_map) = world.get_resource::<NodeEntityMap>() {
                                        found_entity = node_entity_map.get_entity(id);
                                    }
                                    if found_entity.is_none() {
                                        if let Some(loaded) = ref_link.get() {
                                            loaded.clone().spawn(world);
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    if field.is_vec {
                        quote! {
                            for ref_link in &self.#field_name {
                                if let Some(id) = ref_link.id() {
                                    // Check if entity with this id already exists
                                    let mut found_entity = None;
                                    if let Some(node_entity_map) = world.get_resource::<NodeEntityMap>() {
                                        found_entity = node_entity_map.get_entity(id);
                                    }
                                    if found_entity.is_none() {
                                        if let Some(loaded) = ref_link.get() {
                                            loaded.clone().spawn(world);
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        quote! {
                            if let Some(id) = self.#field_name.id() {
                                // Check if entity with this id already exists
                                let mut found_entity = None;
                                if let Some(node_entity_map) = world.get_resource::<NodeEntityMap>() {
                                    found_entity = node_entity_map.get_entity(id);
                                }
                                if found_entity.is_none() {
                                    if let Some(loaded) = self.#field_name.get() {
                                        loaded.clone().spawn(world);
                                    }
                                }
                            }
                        }
                    }
                })
            }
            _ => None,
        });

    quote! {
        impl ClientNode for #struct_name {
            fn spawn(self, world: &mut World) {
                let entity = world.spawn(self.clone()).id();

                // Register id-entity mapping in NodeEntityMap
                if let Some(mut node_entity_map) = world.get_resource_mut::<NodeEntityMap>() {
                    node_entity_map.insert(self.id, entity);
                }

                // Add component links to same entity
                #(#spawn_components)*

                // Create child entities for owned links
                #(#spawn_owned)*

                // Handle ref links with id-entity checking
                #(#spawn_refs)*
            }
        }
    }
}
