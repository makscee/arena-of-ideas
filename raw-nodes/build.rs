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
    let mut named_nodes: Vec<_> = Vec::new();

    for item in syntax_tree.items {
        if let Item::Struct(item_struct) = item {
            let struct_name = item_struct.ident.clone();
            names.push(struct_name.clone());

            // Check if this struct has the #[named_node] attribute
            for attr in &item_struct.attrs {
                if attr.path().is_ident("named_node") {
                    // Validate and get the name field
                    if let Some(_name_field) = get_named_node_field(&item_struct) {
                        named_nodes.push(struct_name.clone());
                    }
                    break;
                }
            }
        }
    }

    let node_kind_enum = generate_node_kind_enum(&names, &named_nodes);

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

fn generate_node_kind_enum(names: &[Ident], named_nodes: &[Ident]) -> TokenStream {
    let named_node_kinds: Vec<_> = named_nodes.iter().collect();

    quote! {
        use std::collections::HashSet;

        #[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, Display, EnumIter, PartialEq, Eq, PartialOrd, Ord, EnumString, AsRefStr, Hash)]
        pub enum NodeKind {
            #[default]
            None,
            #(
                #names,
            )*
        }

        #[derive(Debug, Clone, Copy, Serialize, Deserialize, Display, EnumIter, PartialEq, Eq, PartialOrd, Ord, EnumString, AsRefStr, Hash)]
        pub enum NamedNodeKind {
            #(
                #named_node_kinds,
            )*
        }

        impl NodeKind {
            pub fn named_nodes() -> HashSet<NodeKind> {
                let mut set = HashSet::new();
                #(
                    set.insert(NodeKind::#named_node_kinds);
                )*
                set
            }

            pub fn is_named(self) -> bool {
                Self::named_nodes().contains(&self)
            }
        }

        impl From<NamedNodeKind> for NodeKind {
            fn from(named: NamedNodeKind) -> Self {
                match named {
                    #(
                        NamedNodeKind::#named_node_kinds => NodeKind::#named_node_kinds,
                    )*
                }
            }
        }

        impl TryFrom<NodeKind> for NamedNodeKind {
            type Error = ();

            fn try_from(kind: NodeKind) -> Result<Self, Self::Error> {
                match kind {
                    #(
                        NodeKind::#named_node_kinds => Ok(NamedNodeKind::#named_node_kinds),
                    )*
                    _ => Err(()),
                }
            }
        }

        pub trait NodeKindExt {
            fn to_kind(&self) -> NodeKind;
        }

        impl NodeKindExt for String {
            fn to_kind(&self) -> NodeKind {
                NodeKind::from_str(self).unwrap()
            }
        }

        pub trait NamedNode {
            fn get_name(&self) -> &str;
            fn set_name(&mut self, name: String);
        }
    }
}

fn get_named_node_field(item_struct: &ItemStruct) -> Option<Ident> {
    let mut string_fields = Vec::new();

    for field in &item_struct.fields {
        if let Some(field_name) = &field.ident {
            if let Type::Path(type_path) = &field.ty {
                if let Some(segment) = type_path.path.segments.last() {
                    // Check if it's a String field (not NodePart or NodeParts)
                    if segment.ident == "String" {
                        string_fields.push(field_name.clone());
                    }
                }
            }
        }
    }

    if string_fields.len() != 1 {
        panic!(
            "Named node {} must have exactly one String field that is not a NodePart, found: {:?}",
            item_struct.ident, string_fields
        );
    }

    Some(string_fields.into_iter().next().unwrap())
}
