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

    // Format and write
    let formatted_code = format_code(&generated);
    fs::write(&dest_path, formatted_code).expect("Failed to write generated code");
}

fn generate_server_nodes(
    nodes: &[NodeInfo],
    _node_map: &HashMap<String, NodeInfo>,
) -> proc_macro2::TokenStream {
    let node_structs = nodes.iter().map(|node| {
        let struct_name = &node.name;

        // Generate fields
        let fields = node.fields.iter().map(|field| {
            let field_name = &field.name;
            let field_type = generate_field_type(field, "schema");

            quote! {
                pub #field_name: #field_type
            }
        });

        // Generate accessor methods
        let accessors = generate_accessors(node);

        // All nodes get SpacetimeDB derives for server
        let derives = quote! {
            #[derive(Debug, Clone, spacetimedb::SpacetimeType, serde::Serialize, serde::Deserialize)]
        };

        quote! {
            #derives
            pub struct #struct_name {
                pub id: Option<u64>,
                #(#fields,)*
            }

            impl #struct_name {
                pub fn new() -> Self {
                    Self {
                        id: None,
                        #(#accessors)*
                    }
                }

                pub fn with_id(mut self, id: u64) -> Self {
                    self.id = Some(id);
                    self
                }
            }

            impl Default for #struct_name {
                fn default() -> Self {
                    Self::new()
                }
            }
        }
    });

    // Generate conversion traits
    let conversions = generate_conversions(nodes);

    quote! {
        use schema::{Node as SchemaNode};
        use schema::{HexColor, Action, Reaction, Material, ShopOffer, UnitActionRange, MagicType, Trigger};

        #(#node_structs)*

        #conversions
    }
}
