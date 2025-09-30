use node_build_utils::*;
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
    let dest_path = Path::new(&out_dir).join("server_nodes.rs");

    // Read the raw nodes file from schema
    let input =
        fs::read_to_string("../schema/src/raw_nodes.rs").expect("Failed to read raw_nodes.rs");
    let nodes = parse_nodes_file(&input).expect("Failed to parse raw_nodes.rs");

    let mut node_map = HashMap::new();
    for node in &nodes {
        node_map.insert(node.name.to_string(), node.clone());
    }

    // Generate server-specific node implementations
    let generated = generate_server_nodes(&nodes, &node_map);

    // Format and write
    let formatted_code = format_code(&generated);
    fs::write(&dest_path, formatted_code).expect("Failed to write generated code");
}

fn generate_server_nodes(
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

        // Generate ServerNode implementation
        let server_node_impl = generate_server_node_impl(node, node_map);

        // Generate link loading methods
        let link_methods = generate_link_methods(node, format_ident!("ServerContext"), None);

        // All nodes get SpacetimeDB derives for server
        let derives = quote! {
            #[derive(Debug, Serialize, Deserialize)]
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

            #server_node_impl

            #default_impl
        }
    });

    // Generate conversion traits
    let conversions = generate_conversions(nodes);

    quote! {
        #(#node_structs)*

        #conversions
    }
}

fn generate_server_node_impl(
    node: &NodeInfo,
    _node_map: &HashMap<String, NodeInfo>,
) -> proc_macro2::TokenStream {
    let struct_name = &node.name;

    // Generate save implementation that handles linked fields
    let save_fields = node
        .fields
        .iter()
        .filter_map(|field| match field.link_type {
            LinkType::Component => {
                let field_name = &field.name;
                if field.is_optional {
                    if field.is_vec {
                        Some(quote! {
                            for item in &self.#field_name {
                                if let Some(component) = item.as_ref() {
                                    if let Some(loaded) = component.get() {
                                        loaded.save(ctx);
                                    }
                                }
                            }
                        })
                    } else {
                        Some(quote! {
                            if let Some(component) = &self.#field_name {
                                if let Some(loaded) = component.get() {
                                    loaded.save(ctx);
                                }
                            }
                        })
                    }
                } else {
                    if field.is_vec {
                        Some(quote! {
                            for component in &self.#field_name {
                                component.save(ctx);
                            }
                        })
                    } else {
                        Some(quote! {
                            if let Some(loaded) = self.#field_name.get() {
                                loaded.save(ctx);
                            }
                        })
                    }
                }
            }
            LinkType::Owned => {
                let field_name = &field.name;
                if field.is_optional {
                    if field.is_vec {
                        Some(quote! {
                            for item in &self.#field_name {
                                if let Some(owned) = item.as_ref() {
                                    if let Some(loaded) = owned.get() {
                                        loaded.save(ctx);
                                    }
                                }
                            }
                        })
                    } else {
                        Some(quote! {
                            if let Some(owned) = &self.#field_name {
                                if let Some(loaded) = owned.get() {
                                    loaded.save(ctx);
                                }
                            }
                        })
                    }
                } else {
                    if field.is_vec {
                        Some(quote! {
                            for owned in &self.#field_name {
                                owned.save(ctx);
                            }
                        })
                    } else {
                        Some(quote! {
                            if let Some(loaded) = self.#field_name.get() {
                                loaded.save(ctx);
                            }
                        })
                    }
                }
            }
            LinkType::Ref => {
                let field_name = &field.name;
                if field.is_optional {
                    if field.is_vec {
                        Some(quote! {
                            for item in &self.#field_name {
                                if let Some(ref_link) = item.as_ref() {
                                    if let Some(loaded) = ref_link.get() {
                                        loaded.save(ctx);
                                    }
                                }
                            }
                        })
                    } else {
                        Some(quote! {
                            if let Some(ref_link) = &self.#field_name {
                                if let Some(loaded) = ref_link.get() {
                                    loaded.save(ctx);
                                }
                            }
                        })
                    }
                } else {
                    if field.is_vec {
                        Some(quote! {
                            for ref_link in &self.#field_name {
                                ref_link.save(ctx);
                            }
                        })
                    } else {
                        Some(quote! {
                            if let Some(loaded) = self.#field_name.get() {
                                loaded.save(ctx);
                            }
                        })
                    }
                }
            }
            LinkType::None => None,
        });

    quote! {
        impl ServerNode for #struct_name {
            fn save(&self, ctx: &ReducerContext) {
                // Save linked fields first
                #(#save_fields)*

                // Insert or update this node
                if self.id == 0 {
                    panic!("Node id not set before save");
                }

                let node = self.to_tnode();
                match ctx.db.nodes_world().id().find(self.id) {
                    Some(_) => {
                        // Update existing node
                        ctx.db.nodes_world().id().update(node);
                    }
                    None => {
                        // Insert new node
                        match ctx.db.nodes_world().try_insert(node) {
                            Ok(_) => {}
                            Err(e) => error!("Insert of node {} failed: {}", self.id, e),
                        }
                    }
                }
            }

            fn clone_self(&self, ctx: &ReducerContext, owner: u64) -> Self {
                todo!()
            }

            fn clone(&self, ctx: &ReducerContext, owner: u64) -> Self {
                self.clone_self(ctx, owner)
            }
        }
    }
}
