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

        // Generate ServerNode implementation
        let server_node_impl = generate_server_node_impl(node, node_map);

        // Generate link loading methods
        let link_methods = generate_link_methods(node, "ServerContext");

        let load_methods = generate_load_functions(node, "ServerContext");

        // Generate collect methods
        let collect_owned_ids_method = generate_collect_owned_ids_impl(node);
        let collect_owned_links_method = generate_collect_owned_links_impl(node);

        // All nodes get SpacetimeDB derives for server
        let allow_attrs = generated_code_allow_attrs();
        let derives = quote! {
            #[derive(Debug, Clone)]
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

    // Generate NamedNode trait and implementations
    let named_node_trait = generate_named_node_trait();

    // Generate named node kind match macro
    let named_node_kind_match_macro = generate_named_node_kind_match_macro(nodes);

    // Generate node kind match macro
    let node_kind_match_macro = generate_node_kind_match_macro(nodes);

    // Generate NamedNode implementations for named nodes
    let named_node_impls = nodes
        .iter()
        .filter(|node| node.is_named)
        .map(|node| generate_named_node_impl(node));

    // Generate module
    quote! {
        #(#node_structs)*

        #conversions

        #named_node_trait

        #node_kind_match_macro

        #named_node_kind_match_macro

        #(#named_node_impls)*
    }
}

fn generate_server_node_impl(
    node: &NodeInfo,
    _node_map: &HashMap<String, NodeInfo>,
) -> proc_macro2::TokenStream {
    let struct_name = &node.name;
    let save_method = generate_save_impl(node, "ServerContext");

    let allow_attrs = generated_code_allow_attrs();
    quote! {
        #allow_attrs
        impl ServerNode for #struct_name {
            #save_method
        }
    }
}
