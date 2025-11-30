use node_build_utils::*;

use convert_case::{Case, Casing};
use quote::format_ident;
use quote::quote;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=../schema/src/raw_nodes.rs");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/stdb");

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("client_nodes.rs");
    let reducers_path = Path::new(&out_dir).join("generated_reducers.rs");

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

    // Generate reducer registry
    let reducers_code = generate_reducer_registry();
    fs::write(&reducers_path, reducers_code).expect("Failed to write generated reducers");
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

        // Generate with_* and *_clear() methods
        let with_methods = generate_with_methods(node);

        // Generate default implementation
        let default_impl = generate_default_impl(node);

        // Generate ClientNode implementation
        let client_node_impl = generate_client_node_impl(node);

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
            }

            #serialize_impl

            #allow_attrs
            impl #struct_name {
                #new_method

                #with_methods

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

    // Generate NodeKind spawn extension
    let node_kind_spawn_impl = generate_node_kind_spawn_impl(nodes);

    // Generate NamedNode trait and implementations
    let named_node_trait = generate_named_node_trait();

    // Generate named node kind match macro
    let named_node_kind_match_macro = generate_named_node_kind_match_macro(nodes);

    // Generate node kind match macro
    let node_kind_match_macro = generate_node_kind_match_macro(nodes);

    // Generate FEdit implementations
    let fedit_impls = nodes.iter().map(|node| generate_fedit_impl(node));

    // Generate FRecursiveNodeEdit implementations
    let frecursive_impls = nodes.iter().map(|node| generate_frecursive_impl(node));

    // Generate ToCstr and FDisplay implementations
    let named_node_impls = nodes
        .iter()
        .filter(|node| node.is_named)
        .map(|node| generate_named_node_impl(node));

    let unit_check_functions = generate_unit_check_functions("ClientContext");

    quote! {
        #(#node_structs)*

        #conversions

        #allow_attrs
        #(#tocstr_impls)*

        #allow_attrs
        #(#fedit_impls)*

        #allow_attrs
        #(#frecursive_impls)*

        #node_kind_spawn_impl

        #named_node_trait

        #node_kind_match_macro

        #named_node_kind_match_macro

        #(#named_node_impls)*

        #unit_check_functions
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
                    if let Ok(node) = self.#field_name.take_loaded() {
                        node.clone().spawn(ctx, Some(entity)).track()?;
                        ctx.add_link(self.id, node.id).track()?;
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
                    if let Ok(node) = self.#field_name.take_loaded() {
                        node.clone().spawn(ctx, Some(entity)).track()?;
                        ctx.add_link(self.id, node.id).track()?;
                    }
                })
            }
            LinkType::OwnedMultiple => {
                let field_name = &field.name;
                Some(quote! {
                    if let Ok(nodes) = self.#field_name.take_loaded() {
                        for node in nodes {
                            node.clone().spawn(ctx, None).track()?;
                            ctx.add_link(self.id, node.id).track()?;
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
                    if let Ok(node) = self.#field_name.get() {
                        ctx.add_link(self.id, node.id).track()?;
                    }
                })
            }
            LinkType::RefMultiple => {
                let field_name = &field.name;
                Some(quote! {
                    if let Ok(nodes) = self.#field_name.get() {
                        for node in nodes {
                            ctx.add_link(self.id, node.id).track()?;
                        }
                    }
                })
            }
            _ => None,
        });

    let load_methods = generate_load_functions(node, "ClientContext");

    let allow_attrs = generated_code_allow_attrs();
    quote! {
        #allow_attrs
        impl ClientNode for #struct_name {
            fn spawn(mut self, ctx: &mut ClientContext, entity: Option<Entity>) -> NodeResult<()> {
                if self.id == 0 {
                    panic!("Tried to spawn node without id");
                }
                let entity = match entity {
                    Some(e) => e,
                    None => ctx.world_mut()?.spawn_empty().id(),
                };
                ctx.add_id_entity_link(self.kind(), self.id, entity).track()?;
                #(#spawn_components)*
                #(#spawn_owned)*
                #(#spawn_refs)*
                let kind = Self::kind_s();
                let id = self.id;
                ctx.world_mut().track()?.entity_mut(entity).insert(self);
                kind.on_spawn(ctx, id)
            }


            #load_methods
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

fn generate_frecursive_impl(node: &NodeInfo) -> proc_macro2::TokenStream {
    let struct_name = &node.name;

    let linked_field_calls = node
        .fields
        .iter()
        .filter(|field| !matches!(field.link_type, LinkType::None))
        .map(|field| {
            let field_name = &field.name;
            let field_label = field.name.to_string();
            let target_type = format_ident!("{}", field.target_type);

            match field.link_type {
                LinkType::Component | LinkType::Owned | LinkType::Ref => {
                    quote! {
                        changed |= ui.render_single_link(#field_label, &mut self.#field_name, self.id);
                        if NodeKind::#target_type.is_compact() && self.#field_name.is_loaded() {
                            if let Ok(loaded) = self.#field_name.get_mut() {
                                changed |= loaded.edit(ui).changed();
                            }
                        }
                    }
                }
                LinkType::OwnedMultiple | LinkType::RefMultiple => {
                    quote! {
                        changed |= ui.render_multiple_link(#field_label, &mut self.#field_name, self.id);
                    }
                }
                LinkType::None => unreachable!(),
            }
        })
        .collect::<Vec<_>>();

    let recursive_search_calls = node
        .fields
        .iter()
        .filter(|field| matches!(field.link_type, LinkType::Component | LinkType::Owned | LinkType::OwnedMultiple))
        .map(|field| {
            let field_name = &field.name;
            let field_label = field.name.to_string();

            match field.link_type {
                LinkType::Component | LinkType::Owned => {
                    quote! {
                        if let Ok(loaded) = self.#field_name.get_mut() {
                            if render_node_field_recursive_with_path(ui, #field_label, loaded, breadcrumb_path) {
                                return true;
                            }
                        }
                    }
                }
                LinkType::OwnedMultiple => {
                    quote! {
                        if let Ok(items) = self.#field_name.get_mut() {
                            for (index, item) in items.iter_mut().sorted_by_key(|i| i.id()).enumerate() {
                                let item_field_name = format!("{}#{}", #field_label, index);
                                if render_node_field_recursive_with_path(ui, &item_field_name, item, breadcrumb_path) {
                                    return true;
                                }
                            }
                        }
                    }
                }
                _ => unreachable!(),
            }
        })
        .collect::<Vec<_>>();

    let allow_attrs = generated_code_allow_attrs();
    quote! {
        #allow_attrs
        impl FRecursiveNodeEdit for #struct_name {
            fn render_linked_fields(
                &mut self,
                ui: &mut egui::Ui,
                breadcrumb_path: &mut Vec<crate::ui::NodeBreadcrumb>,
            ) -> bool {
                let mut changed = false;
                #(#linked_field_calls)*
                changed
            }

            fn render_recursive_search(
                &mut self,
                ui: &mut egui::Ui,
                breadcrumb_path: &mut Vec<crate::ui::NodeBreadcrumb>,
            ) -> bool {
                use crate::ui::render::features::render_node_field_recursive_with_path;

                #(#recursive_search_calls)*
                false
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
                        changed = true;
                    }
                });
            }
        });

    let allow_attrs = generated_code_allow_attrs();
    quote! {
        #allow_attrs
        impl FEdit for #struct_name {
            fn edit(&mut self, ui: &mut egui::Ui) -> egui::Response {
                let mut changed = false;
                let mut main_response = ui.group(|ui| {
                    #(#data_field_edits)*
                }).response;

                if changed {
                    main_response.mark_changed();
                }
                main_response
            }
        }
    }
}

fn generate_reducer_registry() -> String {
    let stdb_dir = "src/stdb";
    let mut reducer_names = Vec::new();

    if let Ok(entries) = fs::read_dir(stdb_dir) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    if let Some(filename) = entry.file_name().into_string().ok() {
                        if filename.ends_with("_reducer.rs") {
                            let name = filename.trim_end_matches("_reducer.rs").to_string();
                            reducer_names.push(name);
                        }
                    }
                }
            }
        }
    }

    reducer_names.sort();

    let variants = reducer_names.iter().map(|name| {
        let variant_ident = format_ident!("{}", name.to_case(Case::Pascal));
        quote! { #variant_ident }
    });

    let generated = quote! {
        use strum_macros::EnumIter;
        use strum::IntoEnumIterator;

        /// Auto-generated enum of all reducers from stdb/*_reducer.rs files
        /// Updated by build.rs - do not edit manually
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter)]
        pub enum AllReducers {
            #(#variants),*
        }
    };

    generated.to_string()
}
