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
        impl NodeKind {
            pub fn component_kinds(self) -> HashSet<Self> {
                match self {
                    NodeKind::None => default(),
                    #(
                        Self::#names => {
                            #names::component_kinds()
                        }
                    )*
                }
            }
            pub fn children_kinds(self) -> HashSet<Self> {
                match self {
                    NodeKind::None => default(),
                    #(
                        Self::#names => {
                            #names::children_kinds()
                        }
                    )*
                }
            }
            pub fn get_incubator_links() -> HashMap<Self, HashSet<Self>> {
                let mut links: HashMap<Self, HashSet<Self>> =
                    HashMap::from_iter(Self::iter().map(|k| (k, default())));
                let incubator_children = Incubator::children_kinds();
                for k in &incubator_children {
                    for c in k.children_kinds() {
                        if !incubator_children.contains(&c) {
                            continue;
                        }
                        links.get_mut(&c).unwrap().insert(*k);
                    }
                }
                for (k, l) in &mut links {
                    for c in k.component_kinds() {
                        if !incubator_children.contains(&c) {
                            continue;
                        }
                        l.insert(c);
                    }
                }
                links
            }
        }
        #nodes
    }
    .into()
}
