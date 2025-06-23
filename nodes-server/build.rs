use quote::quote;
use std::env;
use std::fs;
use std::path::Path;
use syn::{Item, parse_file};

fn main() {
    println!("cargo:rerun-if-changed=../nodes/src/raw_nodes.rs");

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("server_impls.rs");

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

            // Add server-specific derives
            let derives = syn::parse_quote!(#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default, Hash, PartialEq, Eq)]);
            item_struct.attrs.insert(0, derives);

            // Make struct public
            item_struct.vis = syn::parse_quote!(pub);

            structs.push(item_struct);
        }
    }

    // Generate server implementations for each struct
    let server_impls: Vec<_> = struct_names
        .iter()
        .map(|name| {
            let name_str = name.to_string();
            quote! {
                impl #name {
                    pub fn new() -> Self {
                        Self::default()
                    }

                    pub fn id(&self) -> u64 {
                        // Server-specific ID generation
                        use std::collections::hash_map::DefaultHasher;
                        use std::hash::{Hash, Hasher};
                        let mut hasher = DefaultHasher::new();
                        self.hash(&mut hasher);
                        hasher.finish()
                    }

                    pub fn persist(&self) -> Result<(), ServerPersistError> {
                        // Server-specific persistence logic
                        Ok(())
                    }

                    pub fn validate(&self) -> Result<(), ServerValidationError> {
                        // Server-specific validation logic
                        Ok(())
                    }
                }

                impl ServerNode for #name {
                    fn serialize_for_client(&self) -> Vec<u8> {
                        // Server-specific serialization for client sync
                        serde_json::to_vec(self).unwrap_or_default()
                    }

                    fn update_from_client(&mut self, _data: &[u8]) -> Result<(), ServerUpdateError> {
                        // Server-specific update logic from client
                        Ok(())
                    }

                    fn authorize_access(&self, _user_id: u64) -> bool {
                        // Server-specific authorization logic
                        true
                    }

                    fn get_dependencies(&self) -> Vec<u64> {
                        // Server-specific dependency tracking
                        Vec::new()
                    }
                }

                impl DatabaseNode for #name {
                    fn table_name() -> &'static str {
                        #name_str
                    }

                    fn primary_key(&self) -> u64 {
                        self.id()
                    }

                    fn save(&self) -> Result<(), DatabaseError> {
                        // Database save logic
                        Ok(())
                    }

                    fn load(_id: u64) -> Result<Self, DatabaseError> {
                        // Database load logic
                        Ok(Self::default())
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

        // Generate the struct definitions with server-specific derives
        #(#structs)*

        // Generate the NodeKind enum
        #node_kind_enum

        // Generate server-specific implementations
        #(#server_impls)*
    };

    fs::write(&dest_path, output.to_string()).expect("Failed to write server implementations file");
}
