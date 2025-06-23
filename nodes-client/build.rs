use quote::quote;
use std::env;
use std::fs;
use std::path::Path;
use syn::{Item, parse_file};

fn main() {
    println!("cargo:rerun-if-changed=../nodes/src/raw_nodes.rs");

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("client_impls.rs");

    // Read the raw nodes file from the nodes crate
    let input =
        fs::read_to_string("../nodes/src/raw_nodes.rs").expect("Failed to read raw_nodes.rs");
    let syntax_tree = parse_file(&input).expect("Failed to parse raw_nodes.rs");

    let mut structs = Vec::new();
    let mut struct_names = Vec::new();

    // Extract all structs and their names
    for item in syntax_tree.items {
        if let Item::Struct(mut item_struct) = item {
            struct_names.push(item_struct.ident.clone());

            // Add client-specific derives
            let derives = syn::parse_quote!(#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default, Hash, PartialEq, Eq)]);
            item_struct.attrs.insert(0, derives);

            // Make struct public
            item_struct.vis = syn::parse_quote!(pub);

            structs.push(item_struct);
        }
    }

    // Generate client implementations for each struct
    let client_impls: Vec<_> = struct_names
        .iter()
        .map(|name| {
            quote! {
                impl #name {
                    pub fn new() -> Self {
                        Self::default()
                    }

                    pub fn id(&self) -> u64 {
                        // Client-specific ID generation
                        use std::collections::hash_map::DefaultHasher;
                        use std::hash::{Hash, Hasher};
                        let mut hasher = DefaultHasher::new();
                        std::ptr::addr_of!(self).hash(&mut hasher);
                        hasher.finish()
                    }
                }

                impl ClientNode for #name {
                    fn sync_from_server(&mut self, _data: &[u8]) -> Result<(), ClientSyncError> {
                        // Client-specific sync logic
                        Ok(())
                    }

                    fn prepare_for_render(&self) -> RenderData {
                        // Client-specific render preparation
                        RenderData::default()
                    }
                }
            }
        })
        .collect();

    // Generate NodeKind enum
    let node_kind_variants: Vec<_> = struct_names
        .iter()
        .map(|name| {
            quote! { #name }
        })
        .collect();

    let node_kind_enum = quote! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum NodeKind {
            #(#node_kind_variants),*
        }
    };

    // Generate the complete output
    let output = quote! {
        // Import base types from nodes crate
        pub use nodes::{
            ChildComponent, ChildComponents, ParentComponent, ParentComponents,
            ParentLinks, parent_links, HexColor
        };

        // Placeholder types that will be defined elsewhere
        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default, Hash, PartialEq, Eq)]
        pub struct Action;
        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default, Hash, PartialEq, Eq)]
        pub struct Reaction;
        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default, Hash, PartialEq, Eq)]
        pub struct Material;
        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default, Hash, PartialEq, Eq)]
        pub struct CardKind;
        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default, Hash, PartialEq, Eq)]
        pub struct ShopOffer;
        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default, Hash, PartialEq, Eq)]
        pub struct UnitTriggerRef;
        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default, Hash, PartialEq, Eq)]
        pub struct UnitActionRef;

        // Generate the struct definitions with client-specific derives
        #(#structs)*

        // Generate the NodeKind enum
        #node_kind_enum

        // Generate client-specific implementations
        #(#client_impls)*
    };

    fs::write(&dest_path, output.to_string()).expect("Failed to write client implementations file");
}
