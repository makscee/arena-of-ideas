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

        // Generate add_components() method
        let add_components_method = generate_add_components(node);

        // Generate default implementation
        let default_impl = generate_default_impl(node);

        // Generate ClientNode implementation
        let client_node_impl = generate_client_node_impl(node, node_map);

        // Generate link loading methods
        let link_methods = generate_client_link_methods(node);

        // Generate load_components method
        let load_components_method = generate_load_functions(node, "ClientContext");

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

                #add_components_method

                pub fn with_id(mut self, id: u64) -> Self {
                    self.id = id;
                    self
                }

                #link_methods

                #load_components_method
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
                Some(quote! {
                    if let Some(loaded) = self.#field_name.get() {
                        loaded.spawn(ctx, entity)?;
                        ctx.add_link(self.id, loaded.id);
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
                            item.spawn(ctx, child_entity)?;
                            ctx.add_link(self.id, item.id);
                        }
                    }
                } else {
                    quote! {
                        if let Some(loaded) = self.#field_name.get() {
                            let child_entity = ctx.world_mut()?.spawn_empty().id();
                            loaded.clone().spawn(ctx, child_entity);
                            ctx.add_link(self.id, loaded.id);
                        }
                    }
                })
            }
            _ => None,
        });

    // Generate unpack_links implementation
    let unpack_links = node.fields.iter().filter_map(|field| {
        if matches!(
            field.link_type,
            LinkType::Owned | LinkType::Component | LinkType::Ref
        ) {
            let field_name = &field.name;
            let target_type = syn::parse_str::<syn::Ident>(&field.target_type).unwrap();

            if field.is_vec {
                match field.link_type {
                    LinkType::Owned => Some(quote! {
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
                    }),
                    LinkType::Ref => Some(quote! {
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
                    }),
                    _ => None,
                }
            } else {
                match field.link_type {
                    LinkType::Component => Some(quote! {
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
                    }),
                    LinkType::Owned => Some(quote! {
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
                    }),
                    LinkType::Ref => Some(quote! {
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
                    }),
                    _ => None,
                }
            }
        } else {
            None
        }
    });

    quote! {
        impl ClientNode for #struct_name {
            fn spawn(self, ctx: &mut ClientContext, entity: Entity) -> NodeResult<()> {
                if self.id == 0 {
                    panic!("Tried to spawn node without id");
                }
                ctx.add_id_entity_link(self.id, entity);
                #(#spawn_components)*
                #(#spawn_owned)*
                ctx.world_mut()?.entity_mut(entity).insert(self);
            }

            fn unpack_links(&mut self, packed: &PackedNodes) {
                #(#unpack_links)*
            }
        }
    }
}
