use node_build_utils::*;
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

    // Add comprehensive allow attributes at the top
    let allow_attrs = generated_code_allow_attrs();
    let final_code = quote! {
        #allow_attrs
        #generated
    };

    // Format and write
    let formatted_code = format_code(&final_code);
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

        // Generate add_components() method
        let add_components_method = generate_add_components(node);

        // Generate default implementation
        let default_impl = generate_default_impl(node);

        // Generate ServerNode implementation
        let server_node_impl = generate_server_node_impl(node, node_map);

        // Generate link loading methods
        let link_methods = generate_server_link_methods(node);

        let load_methods = generate_load_functions(node, "ServerContext");

        // Generate collect methods
        let collect_owned_ids_method = generate_collect_owned_ids_impl(node);
        let collect_owned_links_method = generate_collect_owned_links_impl(node);

        // All nodes get SpacetimeDB derives for server
        let allow_attrs = generated_code_allow_attrs();
        let derives = quote! {
            #[derive(Debug)]
        };

        // Generate manual Serialize/Deserialize implementation
        let serialize_impl = generate_manual_serialize_impl(node);

        quote! {
            #derives
            pub struct #struct_name {
                pub id: u64,
                pub owner: u64,
                #(#fields,)*
            }

            #serialize_impl

            #allow_attrs
            impl #struct_name {
                #new_method

                #add_components_method

                pub fn with_id(mut self, id: u64) -> Self {
                    self.id = id;
                    self
                }

                #link_methods
                #load_methods
                #collect_owned_ids_method
                #collect_owned_links_method
            }

            #server_node_impl

            #default_impl
        }
    });

    // Generate conversion traits
    let conversions = generate_node_impl(nodes);

    // Generate NodeLoader implementation for ServerContext
    let node_loader_impl = generate_node_loader_impl(nodes);

    quote! {
        #(#node_structs)*

        #conversions

        #node_loader_impl
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
                Some(if field.is_vec {
                    quote! {
                        for component in &self.#field_name {
                            component.save(source);
                        }
                    }
                } else {
                    quote! {
                        if let Some(loaded) = self.#field_name.get() {
                            loaded.save(source);
                        }
                    }
                })
            }
            LinkType::Owned => {
                let field_name = &field.name;
                Some(if field.is_vec {
                    quote! {
                        for owned in &self.#field_name {
                            owned.save(source);
                        }
                    }
                } else {
                    quote! {
                        if let Some(loaded) = self.#field_name.get() {
                            loaded.save(source);
                        }
                    }
                })
            }
            LinkType::Ref => {
                let field_name = &field.name;
                Some(if field.is_vec {
                    quote! {
                        for ref_link in &self.#field_name {
                            if let Some(loaded) = ref_link.get() {
                                loaded.save(source);
                            }
                        }
                    }
                } else {
                    quote! {
                        if let Some(loaded) = self.#field_name.get() {
                            loaded.save(source);
                        }
                    }
                })
            }
            LinkType::None => None,
        });

    let allow_attrs = generated_code_allow_attrs();
    quote! {
        #allow_attrs
        impl ServerNode for #struct_name {
            fn save(&self, source: &ServerSource) {
                // Save linked fields first
                #(#save_fields)*

                // Insert or update this node
                if self.id == 0 {
                    panic!("Node id not set before save");
                }

                let node = self.to_tnode();
                let ctx = source.rctx();
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

            fn clone_self(&self, ctx: &ServerContext, owner: u64) -> Self {
                todo!()
            }

            fn clone(&self, ctx: &ServerContext, owner: u64) -> Self {
                self.clone_self(ctx, owner)
            }
        }
    }
}

fn generate_node_loader_impl(nodes: &[NodeInfo]) -> proc_macro2::TokenStream {
    let load_and_get_var_arms = nodes.iter().map(|node| {
        let node_name = &node.name;
        quote! {
            NodeKind::#node_name => {
                let node: #node_name = node_id.load_node(self.rctx())?;
                node.get_var(var)
            }
        }
    });

    let load_and_set_var_arms = nodes.iter().map(|node| {
        let node_name = &node.name;
        quote! {
            NodeKind::#node_name => {
                let mut node: #node_name = node_id.load_node(self.rctx())?;
                node.set_var(var, value)?;
                node.save(self);
                Ok(())
            }
        }
    });

    let allow_attrs = generated_code_allow_attrs();
    quote! {
        #allow_attrs
        impl<'a> NodeLoader for ServerSource<'a> {
            fn load_and_get_var(
                &self,
                node_kind: NodeKind,
                node_id: u64,
                var: VarName,
            ) -> NodeResult<VarValue> {
                match node_kind {
                    #(#load_and_get_var_arms,)*
                    NodeKind::None => Err(NodeError::Custom("Cannot get var from None node".into())),
                }
            }

            fn load_and_set_var(
                &mut self,
                node_kind: NodeKind,
                node_id: u64,
                var: VarName,
                value: VarValue,
            ) -> NodeResult<()> {
                match node_kind {
                    #(#load_and_set_var_arms,)*
                    NodeKind::None => Err(NodeError::Custom("Cannot set var on None node".into())),
                }
            }
        }
    }
}
