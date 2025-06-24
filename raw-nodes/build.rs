use proc_macro2::TokenStream;
use quote::quote;
use std::env;
use std::fs;
use std::path::Path;
use syn::*;

fn main() {
    println!("cargo:rerun-if-changed=src/raw_nodes.rs");
    println!("cargo::rerun-if-changed=build.rs");

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("node_kind.rs");

    // Read the raw nodes file
    let input = fs::read_to_string("src/raw_nodes.rs").expect("Failed to read raw_nodes.rs");
    let syntax_tree = parse_file(&input).expect("Failed to parse raw_nodes.rs");

    let mut names: Vec<_> = Vec::new();
    for item in syntax_tree.items {
        if let Item::Struct(item_struct) = item {
            names.push(item_struct.ident.clone());
        }
    }

    let node_kind_enum = generate_node_kind_enum(names);

    let output = quote! {
        #node_kind_enum
    };

    // Parse the generated code and format it
    let formatted_code = match syn::parse_file(&output.to_string()) {
        Ok(parsed) => prettyplease::unparse(&parsed),
        Err(_) => {
            // If parsing fails, fall back to unformatted output
            eprintln!(
                "Warning: Failed to parse generated code for formatting, using unformatted output"
            );
            output.to_string()
        }
    };

    fs::write(&dest_path, formatted_code).expect("Failed to write NodeKind enum file");
}

fn generate_node_kind_enum(names: Vec<Ident>) -> TokenStream {
    quote! {
        #[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, Display, EnumIter, PartialEq, Eq, PartialOrd, Ord, EnumString, AsRefStr, Hash)]
        pub enum NodeKind {
            #[default]
            None,
            #(
                #names,
            )*
        }

        pub trait NodeKindExt {
            fn to_kind(&self) -> NodeKind;
        }

        impl NodeKindExt for String {
            fn to_kind(&self) -> NodeKind {
                NodeKind::from_str(self).unwrap()
            }
        }
    }
}
