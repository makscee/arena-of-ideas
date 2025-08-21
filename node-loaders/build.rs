use proc_macro2::TokenStream;
use quote::quote;
use std::env;
use std::fs;
use std::path::Path;
use syn::*;

fn main() {
    println!("cargo:rerun-if-changed=../raw-nodes/src/raw_nodes.rs");
    println!("cargo::rerun-if-changed=build.rs");

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("generated_loaders.rs");

    // Read the raw nodes file
    let input =
        fs::read_to_string("../raw-nodes/src/raw_nodes.rs").expect("Failed to read raw_nodes.rs");
    let syntax_tree = parse_file(&input).expect("Failed to parse raw_nodes.rs");

    let mut structs = Vec::new();
    for item in syntax_tree.items {
        if let Item::Struct(item_struct) = item {
            structs.push(item_struct);
        }
    }

    let loader_structs = generate_loader_structs(&structs);

    let output = quote! {
        #loader_structs
    };

    // Parse and write generated loaders
    let formatted_code = match syn::parse_file(&output.to_string()) {
        Ok(parsed) => prettyplease::unparse(&parsed),
        Err(_) => {
            eprintln!(
                "Warning: Failed to parse generated loader code for formatting, using unformatted output"
            );
            output.to_string()
        }
    };

    fs::write(&dest_path, formatted_code).expect("Failed to write generated loaders file");
}

fn generate_loader_structs(structs: &[ItemStruct]) -> TokenStream {
    let loaders = structs.iter().map(|item_struct| {
        let struct_name = &item_struct.ident;
        let loader_name = quote::format_ident!("{}Loader", struct_name);

        let pnf = schema::parse_node_fields(&item_struct.fields);
        let (one_fields, _one_types) = pnf.one_owned();
        let (many_fields, _many_types) = pnf.many_owned();

        let all_part_fields = one_fields.iter().chain(many_fields.iter());

        let flag_fields = all_part_fields.clone().map(|field| {
            let flag_name = quote::format_ident!("{}_load", field);
            quote! { pub #flag_name: bool }
        });

        let with_methods = all_part_fields.clone().map(|field| {
            let method_name = quote::format_ident!("with_{}", field);
            let without_method_name = quote::format_ident!("without_{}", field);
            let flag_name = quote::format_ident!("{}_load", field);
            quote! {
                pub fn #method_name(mut self) -> Self {
                    self.#flag_name = true;
                    self
                }

                pub fn #without_method_name(mut self) -> Self {
                    self.#flag_name = false;
                    self
                }
            }
        });

        let default_flags = all_part_fields.clone().map(|field| {
            let flag_name = quote::format_ident!("{}_load", field);
            quote! { #flag_name: false }
        });

        let set_all_flags = all_part_fields.clone().map(|field| {
            let flag_name = quote::format_ident!("{}_load", field);
            quote! { self.#flag_name = true; }
        });

        quote! {
            #[derive(Debug, Clone)]
            pub struct #loader_name {
                pub id: u64,
                #(#flag_fields,)*
            }

            impl #loader_name {
                pub fn new(id: u64) -> Self {
                    Self {
                        id,
                        #(#default_flags,)*
                    }
                }

                pub fn with_all_parts(mut self) -> Self {
                    #(#set_all_flags)*
                    self
                }

                #(#with_methods)*
            }
        }
    });

    quote! {
        #(#loaders)*
    }
}
