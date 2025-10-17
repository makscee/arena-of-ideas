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

        // Add is_dirty field for change tracking
        let is_dirty_field = quote! {
            pub is_dirty: bool
        };

        // Generate new() method with parameters
        let new_method = generate_new(node);

        // Generate with_* and *_clear() methods
        let with_methods = generate_with_methods(node);

        // Generate default implementation
        let default_impl = generate_default_impl(node);

        // Generate ClientNode implementation
        let client_node_impl = generate_client_node_impl(node);

        // Generate link loading methods
        let link_methods = generate_link_methods(node, "ClientContext");

        let load_components_method = generate_load_functions(node, "ClientContext");

        // Generate collect methods
        let collect_owned_ids_method = generate_collect_owned_ids_impl(node);
        let collect_owned_links_method = generate_collect_owned_links_impl(node);

        // All nodes are Components in client
        let allow_attrs = generated_code_allow_attrs();
        let derives = quote! {
            #allow_attrs
            #[derive(Debug, Clone, BevyComponent)]
        };

        // Generate manual Serialize/Deserialize implementation
        let serialize_impl = generate_manual_serialize_impl(node);

        quote! {
            #derives
            pub struct #struct_name {
                pub id: u64,
                pub owner: u64,
                #(#fields,)*
                #is_dirty_field
            }

            #serialize_impl

            #allow_attrs
            impl #struct_name {
                #new_method

                #with_methods

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

    // Generate NodeKind spawn extension
    let node_kind_spawn_impl = generate_node_kind_spawn_impl(nodes);

    // Generate NamedNode trait and implementations
    let named_node_trait = quote! {
        pub trait NamedNode {
            fn named_kind() -> NamedNodeKind;
        }
    };

    // Generate FEdit implementations
    let fedit_impls = nodes.iter().map(|node| generate_fedit_impl(node));

    // Generate ToCstr and FDisplay implementations
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

        #allow_attrs
        #(#fedit_impls)*

        #node_loader_impl

        #node_kind_spawn_impl

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
                        loaded.clone().spawn(ctx, Some(entity)).track()?;
                        ctx.add_link(self.id, loaded.id).track()?;
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
                Some(quote! {
                    if let Some(loaded) = self.#field_name.get() {
                        loaded.clone().spawn(ctx, None).track()?;
                        ctx.add_link(self.id, loaded.id).track()?;
                    }
                })
            }
            LinkType::OwnedMultiple => {
                let field_name = &field.name;
                Some(quote! {
                    if let Some(items) = self.#field_name.get() {
                        for item in items {
                            item.clone().spawn(ctx, None).track()?;
                            ctx.add_link(self.id, item.id).track()?;
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
                Some(quote! {
                    if let Some(ref_id) = self.#field_name.id() {
                        ctx.add_link(self.id, ref_id).track()?;
                    }
                })
            }
            LinkType::RefMultiple => {
                let field_name = &field.name;
                Some(quote! {
                    if let Some(ids) = self.#field_name.ids() {
                        for &ref_id in &ids {
                            ctx.add_link(self.id, ref_id).track()?;
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
            fn spawn(self, ctx: &mut ClientContext, entity: Option<Entity>) -> NodeResult<()> {
                if self.id == 0 {
                    panic!("Tried to spawn node without id");
                }
                let entity = match entity {
                    Some(e) => e,
                    None => ctx.world_mut()?.spawn_empty().id(),
                };
                ctx.add_id_entity_link(self.id, entity).track()?;
                #(#spawn_components)*
                #(#spawn_owned)*
                #(#spawn_refs)*
                let kind = self.kind();
                let id = self.id;
                ctx.world_mut().track()?.entity_mut(entity).insert(self);
                kind.on_spawn(ctx, id)
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
                Err(NodeError::not_found(node_id))
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
                Err(NodeError::not_found(node_id))
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
                    NodeKind::None => Err(NodeError::custom("Cannot get var from None node")),
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
                    NodeKind::None => Err(NodeError::custom("Cannot set var on None node")),
                }
            }
        }
    }
}

fn generate_node_kind_spawn_impl(nodes: &[NodeInfo]) -> proc_macro2::TokenStream {
    let spawn_arms = nodes.iter().map(|node| {
        let node_name = &node.name;
        quote! {
            NodeKind::#node_name => node.to_node::<#node_name>()?.spawn(ctx, None)
        }
    });

    let allow_attrs = generated_code_allow_attrs();
    quote! {
        pub trait NodeKindSpawnExt {
            fn spawn(self, ctx: &mut ClientContext, node: &TNode) -> NodeResult<()>;
        }

        #allow_attrs
        impl NodeKindSpawnExt for NodeKind {
            fn spawn(self, ctx: &mut ClientContext, node: &TNode) -> NodeResult<()> {
                match self {
                    #(#spawn_arms,)*
                    NodeKind::None => Err(NodeError::custom("Cannot spawn None node")),
                }
            }
        }
    }
}

fn generate_fedit_impl(node: &NodeInfo) -> proc_macro2::TokenStream {
    let struct_name = &node.name;

    // Generate edits for data fields (non-link fields)
    let data_field_edits = node
        .fields
        .iter()
        .filter(|field| matches!(field.link_type, LinkType::None))
        .map(|field| {
            let field_name = &field.name;
            let field_label = field.name.to_string();
            quote! {
                ui.horizontal(|ui| {
                    ui.label(#field_label);
                    let field_response = self.#field_name.edit(ui);
                    if field_response.changed() {
                        self.is_dirty = true;
                        changed = true;
                    }
                    field_response
                }).inner;
            }
        });

    // Generate edits for linked nodes
    let link_field_edits = node
        .fields
        .iter()
        .filter(|field| !matches!(field.link_type, LinkType::None))
        .map(|field| {
            let field_name = &field.name;
            let field_label = field.name.to_string();

            match field.link_type {
                LinkType::Component | LinkType::Owned => {
                    quote! {
                        ui.collapsing(#field_label, |ui| {
                            ui.horizontal(|ui| {
                                if self.#field_name.get().is_some() {
                                    if ui.button("Delete").clicked() {
                                        self.#field_name = Default::default();
                                        self.is_dirty = true;
                                        changed = true;
                                    }
                                } else {
                                    ui.label("(no inner node)");
                                }
                            });

                            if let Some(inner_node) = self.#field_name.get() {
                                ui.label("Inner node present (editing not yet implemented)");
                            }
                        });
                    }
                }
                LinkType::OwnedMultiple => {
                    quote! {
                        ui.collapsing(#field_label, |ui| {
                            if let Some(items) = self.#field_name.get() {
                                ui.label(format!("{} items", items.len()));
                                for (index, _item) in items.iter().enumerate() {
                                    ui.horizontal(|ui| {
                                        ui.label(format!("Item {}", index));
                                    });
                                }
                            } else {
                                ui.label("(no items)");
                            }
                        });
                    }
                }
                LinkType::Ref => {
                    quote! {
                        ui.horizontal(|ui| {
                            ui.label(#field_label);
                            if let Some(id) = self.#field_name.id() {
                                ui.label(format!("ID: {}", id));
                                if ui.button("Clear").clicked() {
                                    self.#field_name = Default::default();
                                    self.is_dirty = true;
                                    changed = true;
                                }
                            } else {
                                ui.label("(no reference)");
                            }
                        });
                    }
                }
                LinkType::RefMultiple => {
                    quote! {
                        ui.collapsing(#field_label, |ui| {
                            if let Some(ids) = self.#field_name.ids() {
                                ui.label(format!("{} references", ids.len()));
                                for (index, id) in ids.iter().enumerate() {
                                    ui.horizontal(|ui| {
                                        ui.label(format!("ID {}: {}", index, id));
                                    });
                                }
                            } else {
                                ui.label("(no references)");
                            }
                        });
                    }
                }
                LinkType::None => unreachable!(),
            }
        });

    let allow_attrs = generated_code_allow_attrs();
    quote! {
        #allow_attrs
        impl FEdit for #struct_name {
            fn edit(&mut self, ui: &mut egui::Ui) -> egui::Response {
                let mut changed = false;

                let mut main_response = ui.vertical(|ui| {
                    ui.group(|ui| {
                        ui.label(format!("Node ID: {}", self.id));
                        ui.label(format!("Owner: {}", self.owner));
                        if self.is_dirty {
                            ui.colored_label(egui::Color32::YELLOW, "Modified");
                        }
                    });

                    ui.separator();

                    #(#data_field_edits)*
                    #(#link_field_edits)*

                    ui.label("")
                }).inner;

                if changed {
                    main_response.mark_changed();
                    main_response
                } else {
                    main_response
                }
            }
        }
    }
}
