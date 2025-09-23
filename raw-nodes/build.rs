use proc_macro2::TokenStream;
use quote::quote;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use syn::*;

#[derive(Debug, Clone, PartialEq)]
enum NodeType {
    Named,
    Content,
    System,
}

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
    let mut content_nodes: Vec<_> = Vec::new();
    let mut system_nodes: Vec<_> = Vec::new();
    let mut cardinality_map: HashMap<String, Vec<String>> = HashMap::new();

    for item in syntax_tree.items {
        if let Item::Struct(item_struct) = item {
            let struct_name = item_struct.ident.clone();
            names.push(struct_name.clone());

            // Check for node type attributes
            let mut node_type = None;
            for attr in &item_struct.attrs {
                if attr.path().is_ident("node") {
                    if let Ok(Meta::List(meta_list)) = attr.meta.clone().try_into() {
                        let type_str = meta_list.tokens.to_string();
                        node_type = match type_str.as_str() {
                            "named" => Some(NodeType::Named),
                            "content" => Some(NodeType::Content),
                            "system" => Some(NodeType::System),
                            _ => None,
                        };
                    }
                    break;
                }
            }

            // Categorize nodes by type
            match node_type {
                Some(NodeType::Named) => named_nodes.push(struct_name.clone()),
                Some(NodeType::Content) => content_nodes.push(struct_name.clone()),
                Some(NodeType::System) => system_nodes.push(struct_name.clone()),
                None => panic!("Node has to have a type"),
            }

            // Parse link_cardinality attributes
            parse_cardinality_attributes(&item_struct, &mut cardinality_map);
        }
    }

    let node_kind_enum = generate_node_kind_enum(
        &names,
        &named_nodes,
        &content_nodes,
        &system_nodes,
        &cardinality_map,
    );

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

fn generate_node_kind_enum(
    names: &[Ident],
    named_nodes: &[Ident],
    content_nodes: &[Ident],
    system_nodes: &[Ident],
    cardinality_map: &HashMap<String, Vec<String>>,
) -> TokenStream {
    let named_node_kinds: Vec<_> = named_nodes.iter().collect();
    let content_node_kinds: Vec<_> = content_nodes.iter().collect();
    let system_node_kinds: Vec<_> = system_nodes.iter().collect();

    // Generate cardinality functions
    let cardinality_functions = generate_cardinality_functions(cardinality_map);

    quote! {
        use std::collections::HashSet;

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub enum NodeType {
            Named,
            Content,
            System,
        }

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

            pub fn content_nodes() -> HashSet<NodeKind> {
                let mut set = HashSet::new();
                #(
                    set.insert(NodeKind::#content_node_kinds);
                )*
                set
            }

            pub fn system_nodes() -> HashSet<NodeKind> {
                let mut set = HashSet::new();
                #(
                    set.insert(NodeKind::#system_node_kinds);
                )*
                set
            }

            pub fn is_named(self) -> bool {
                Self::named_nodes().contains(&self)
            }

            pub fn is_content(self) -> bool {
                Self::content_nodes().contains(&self)
            }

            pub fn is_system(self) -> bool {
                Self::system_nodes().contains(&self)
            }

            pub fn node_type(self) -> NodeType {
                if self.is_named() {
                    NodeType::Named
                } else if self.is_content() {
                    NodeType::Content
                } else {
                    NodeType::System
                }
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
        impl NodeKindExt for NamedNodeKind {
            fn to_kind(&self) -> NodeKind {
                match self {
                    #(
                        Self::#named_node_kinds => NodeKind::#named_node_kinds,
                    )*

                }
            }
        }

        pub trait NamedNode {
            fn get_name(&self) -> &str;
            fn set_name(&mut self, name: String);
        }

        #cardinality_functions
    }
}

fn parse_cardinality_attributes(
    item_struct: &ItemStruct,
    cardinality_map: &mut HashMap<String, Vec<String>>,
) {
    let struct_name = item_struct.ident.to_string();

    for field in &item_struct.fields {
        // Check for link_cardinality attribute on the field
        for attr in &field.attrs {
            if attr.path().is_ident("link_cardinality") {
                if let Ok(Meta::List(meta_list)) = attr.meta.clone().try_into() {
                    let cardinality_str = meta_list.tokens.to_string();

                    // Extract the target type from the field
                    if let Some(target_type) = extract_node_part_type(field) {
                        match cardinality_str.as_str() {
                            "one_to_one" => {
                                // Both kinds are sources in one_to_one
                                cardinality_map
                                    .entry(struct_name.clone())
                                    .or_insert_with(Vec::new)
                                    .push(target_type.clone());
                                cardinality_map
                                    .entry(target_type)
                                    .or_insert_with(Vec::new)
                                    .push(struct_name.clone());
                            }
                            "one_to_many" => {
                                // The "Many" kind (target) is the source
                                cardinality_map
                                    .entry(target_type)
                                    .or_insert_with(Vec::new)
                                    .push(struct_name.clone());
                            }
                            "many_to_one" => {
                                // The "Many" kind (struct) is the source
                                cardinality_map
                                    .entry(struct_name.clone())
                                    .or_insert_with(Vec::new)
                                    .push(target_type);
                            }
                            _ => panic!("Wrong link cardinality {}", cardinality_str.as_str()),
                        }
                    }
                }
            }
        }
    }
}

fn extract_node_part_type(field: &Field) -> Option<String> {
    if let Type::Path(type_path) = &field.ty {
        if let Some(segment) = type_path.path.segments.last() {
            if segment.ident == "NodePart" || segment.ident == "NodeParts" {
                if let PathArguments::AngleBracketed(args) = &segment.arguments {
                    // Skip the first argument (Parent/Child) and get the second (the target type)
                    if args.args.len() >= 2 {
                        if let GenericArgument::Type(Type::Path(target_path)) = &args.args[1] {
                            if let Some(target_segment) = target_path.path.segments.last() {
                                return Some(target_segment.ident.to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

fn generate_cardinality_functions(cardinality_map: &HashMap<String, Vec<String>>) -> TokenStream {
    let mut source_functions = Vec::new();

    for name in cardinality_map.keys() {
        let node_kind = syn::Ident::new(name, proc_macro2::Span::call_site());
        let sources = cardinality_map.get(name).unwrap();
        let source_kinds: Vec<_> = sources
            .iter()
            .map(|s| {
                let source_kind = syn::Ident::new(s, proc_macro2::Span::call_site());
                quote! { NodeKind::#source_kind }
            })
            .collect();

        source_functions.push(quote! {
            NodeKind::#node_kind => {
                let mut set = HashSet::new();
                #(set.insert(#source_kinds);)*
                set
            }
        });
    }

    quote! {
        impl NodeKind {
            pub fn source_of(self) -> HashSet<NodeKind> {
                use std::collections::HashSet;
                match self {
                    #(#source_functions)*
                    _ => HashSet::new(),
                }
            }
        }
    }
}
