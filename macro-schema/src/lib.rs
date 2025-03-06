use darling::FromMeta;
use itertools::Itertools;
use proc_macro::TokenStream;
use quote::quote;
use syn::*;

#[proc_macro]
pub fn nodes(_: TokenStream) -> TokenStream {
    let nodes = include_str!("../../schema/src/nodes.rs");
    let mut names = Vec::default();
    for (i, _) in nodes.match_indices("struct") {
        let mut name = String::new();
        let mut k = i + 7;
        while let Some(c) = nodes.chars().nth(k) {
            if !c.is_alphabetic() {
                break;
            }
            name.push(c);
            k += 1;
        }
        names.push(name);
    }
    let names = names
        .into_iter()
        .map(|n| Ident::from_string(&n).unwrap())
        .collect_vec();
    let nodes: proc_macro2::TokenStream = nodes
        .replace("struct", "#[node] pub struct")
        .parse()
        .unwrap();
    quote! {
        pub trait GetNodeKind {
            fn kind(&self) -> NodeKind;
        }
        pub trait GetNodeKindSelf {
            fn kind_s() -> NodeKind;
        }
        #[derive(Debug, Default, Clone, Copy, Display, EnumIter, PartialEq, Eq, strum_macros::EnumString, strum_macros::AsRefStr, Hash)]
        #[node_kinds]
        pub enum NodeKind {
            #[default]
            None,
            #(#names,)*
        }
        #(
            impl GetNodeKind for #names {
                fn kind(&self) -> NodeKind {
                    NodeKind::#names
                }
            }
            impl GetNodeKindSelf for #names {
                fn kind_s() -> NodeKind {
                    NodeKind::#names
                }
            }
        )*
        #nodes
    }
    .into()
}
