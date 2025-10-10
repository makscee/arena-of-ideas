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
    let generated = generate_client_nodes(&nodes);

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

fn generate_client_nodes(nodes: &[NodeInfo]) -> proc_macro2::TokenStream {
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

        // Generate ClientNode implementation
        let client_node_impl = generate_client_node_impl(node);

        // Generate link loading methods
        let link_methods = generate_client_link_methods(node);

        let load_components_method = generate_load_functions(node, "ClientContext");

        // Generate collect methods
        let collect_owned_ids_method = generate_collect_owned_ids_impl(node);
        let collect_owned_links_method = generate_collect_owned_links_impl(node);

        // All nodes are Components in client
        let allow_attrs = generated_code_allow_attrs();
        let derives = quote! {
            #allow_attrs
            #[derive(Debug, Clone, BevyComponent, Serialize, Deserialize)]
        };

        quote! {
            #derives
            pub struct #struct_name {
                pub id: u64,
                pub owner: u64,
                #(#fields,)*
            }

            #allow_attrs
            impl #struct_name {
                #new_method

                #add_components_method

                pub fn with_id(mut self, id: u64) -> Self {
                    self.id = id;
                    self
                }

                #link_methods

                #load_components_method
                #collect_owned_ids_method
                #collect_owned_links_method
            }

            #client_node_impl

            #default_impl
        }
    });

    // Generate conversion traits
    let allow_attrs = generated_code_allow_attrs();
    let conversions = generate_node_impl(nodes);

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

    // Generate NodeLoader implementation for ClientContext
    let node_loader_impl = generate_node_loader_impl(nodes);

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
        #(#node_structs)*

        #conversions

        #allow_attrs
        #(#tocstr_impls)*

        #node_loader_impl

        #named_node_trait

        #(#named_node_impls)*
    }
}

fn generate_client_node_impl(node: &NodeInfo) -> proc_macro2::TokenStream {
    let struct_name = &node.name;

    // Generate spawn implementation that handles entity creation and linking
    let spawn_components = node
        .fields
        .iter()
        .filter_map(|field| match field.link_type {
            LinkType::Component => {
                let field_name = &field.name;
                Some(quote! {
                    if let Some(loaded) = self.#field_name.get() {
                        loaded.clone().spawn(ctx, entity)?;
                        ctx.add_link(self.id, loaded.id)?;
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
                Some(if field.is_vec {
                    quote! {
                        for item in &self.#field_name {
                            let child_entity = ctx.world_mut()?.spawn_empty().id();
                            item.clone().spawn(ctx, child_entity)?;
                            ctx.add_link(self.id, item.id)?;
                        }
                    }
                } else {
                    quote! {
                        if let Some(loaded) = self.#field_name.get() {
                            let child_entity = ctx.world_mut()?.spawn_empty().id();
                            loaded.clone().spawn(ctx, child_entity)?;
                            ctx.add_link(self.id, loaded.id)?;
                        }
                    }
                })
            }
            _ => None,
        });

    let allow_attrs = generated_code_allow_attrs();
    quote! {
        #allow_attrs
        impl ClientNode for #struct_name {
            fn spawn(self, ctx: &mut ClientContext, entity: Entity) -> NodeResult<()> {
                if self.id == 0 {
                    panic!("Tried to spawn node without id");
                }
                ctx.add_id_entity_link(self.id, entity)?;
                #(#spawn_components)*
                #(#spawn_owned)*
                ctx.world_mut()?.entity_mut(entity).insert(self);
                Ok(())
            }
        }
    }
}

fn generate_node_loader_impl(nodes: &[NodeInfo]) -> proc_macro2::TokenStream {
    let load_and_get_var_arms = nodes.iter().map(|node| {
        let node_name = &node.name;
        quote! {
            NodeKind::#node_name => {
                let world = self.world()?;
                if let Some(entity_map) = world.get_resource::<NodeEntityMap>() {
                    if let Some(entity) = entity_map.get_entity(node_id) {
                        if let Some(node) = world.get::<#node_name>(entity) {
                            return node.get_var(var);
                        }
                    }
                }
                Err(NodeError::NotFound(node_id))
            }
        }
    });

    let load_and_set_var_arms = nodes.iter().map(|node| {
        let node_name = &node.name;
        quote! {
            NodeKind::#node_name => {
                let world = self.world_mut()?;
                if let Some(entity_map) = world.get_resource::<NodeEntityMap>() {
                    if let Some(entity) = entity_map.get_entity(node_id) {
                        if let Some(mut node) = world.get_mut::<#node_name>(entity) {
                            node.set_var(var, value)?;
                            return Ok(());
                        }
                    }
                }
                Err(NodeError::NotFound(node_id))
            }
        }
    });

    let allow_attrs = generated_code_allow_attrs();
    quote! {
        #allow_attrs
        impl<'w> NodeLoader for WorldSource<'w> {
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
