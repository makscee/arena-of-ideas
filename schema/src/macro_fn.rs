use proc_macro2::TokenStream;
use quote::quote;
use strum::VariantNames;
use syn::{Fields, GenericArgument, Ident, PathArguments, Type, TypePath};
use syn::{File, ItemStruct};

use crate::VarName;
use crate::VarValue;

#[derive(Default)]
pub struct ParsedNodeFields {
    pub var_fields: Vec<Ident>,
    pub var_types: Vec<Type>,
    pub data_fields: Vec<Ident>,
    pub data_types: Vec<Type>,
    pub all_data_fields: Vec<Ident>,
    pub all_data_types: Vec<Type>,
    pub children_fields: Vec<Ident>,
    pub children_types: Vec<Type>,
    pub parents_fields: Vec<Ident>,
    pub parents_types: Vec<Type>,
    pub child_fields: Vec<Ident>,
    pub child_types: Vec<Type>,
    pub parent_fields: Vec<Ident>,
    pub parent_types: Vec<Type>,
}

impl ParsedNodeFields {
    pub fn one_owned(&self) -> (Vec<Ident>, Vec<Type>) {
        let mut fields = self.child_fields.clone();
        fields.extend(self.parent_fields.clone());
        let mut types = self.child_types.clone();
        types.extend(self.parent_types.clone());
        (fields, types)
    }
    pub fn many_owned(&self) -> (Vec<Ident>, Vec<Type>) {
        let mut fields = self.children_fields.clone();
        fields.extend(self.parents_fields.clone());
        let mut types = self.children_types.clone();
        types.extend(self.parents_types.clone());
        (fields, types)
    }
}

pub fn parse_node_fields(fields: &Fields) -> ParsedNodeFields {
    let mut pnf = ParsedNodeFields::default();
    let ParsedNodeFields {
        var_fields,
        var_types,
        data_fields,
        data_types,
        all_data_fields,
        all_data_types,
        children_fields,
        children_types,
        parents_fields,
        parents_types,
        child_fields,
        child_types,
        parent_fields,
        parent_types,
    } = &mut pnf;
    fn inner_type(type_path: &TypePath) -> Type {
        match &type_path.path.segments.first().unwrap().arguments {
            PathArguments::AngleBracketed(arg) => match arg.args.first().unwrap() {
                GenericArgument::Type(t) => t.clone(),
                _ => unimplemented!(),
            },
            _ => unimplemented!(),
        }
    }
    for field in fields.iter() {
        let ty = &field.ty;
        let field_ident = field.ident.clone().unwrap();
        let field_str = field_ident.to_string();
        match ty {
            syn::Type::Path(type_path) => {
                let type_ident = type_path.path.segments.first().unwrap().ident.to_string();
                if type_ident == "NodeParts" {
                    if let syn::PathArguments::AngleBracketed(angle_args) =
                        &type_path.path.segments.first().unwrap().arguments
                    {
                        if angle_args.args.len() == 2 {
                            if let (
                                syn::GenericArgument::Type(relation_type),
                                syn::GenericArgument::Type(node_type),
                            ) = (&angle_args.args[0], &angle_args.args[1])
                            {
                                if let syn::Type::Path(relation_path) = relation_type {
                                    let relation_ident = relation_path
                                        .path
                                        .segments
                                        .first()
                                        .unwrap()
                                        .ident
                                        .to_string();
                                    if relation_ident == "Parent" {
                                        parents_fields.push(field_ident.clone());
                                        parents_types.push(node_type.clone());
                                    } else if relation_ident == "Child" {
                                        children_fields.push(field_ident.clone());
                                        children_types.push(node_type.clone());
                                    }
                                }
                            }
                        }
                    }
                } else if type_ident == "NodePart" {
                    if let syn::PathArguments::AngleBracketed(angle_args) =
                        &type_path.path.segments.first().unwrap().arguments
                    {
                        if angle_args.args.len() == 2 {
                            if let (
                                syn::GenericArgument::Type(relation_type),
                                syn::GenericArgument::Type(node_type),
                            ) = (&angle_args.args[0], &angle_args.args[1])
                            {
                                if let syn::Type::Path(relation_path) = relation_type {
                                    let relation_ident = relation_path
                                        .path
                                        .segments
                                        .first()
                                        .unwrap()
                                        .ident
                                        .to_string();
                                    if relation_ident == "Parent" {
                                        parent_fields.push(field_ident.clone());
                                        parent_types.push(node_type.clone());
                                    } else if relation_ident == "Child" {
                                        child_fields.push(field_ident.clone());
                                        child_types.push(node_type.clone());
                                    }
                                }
                            }
                        }
                    }
                } else if type_ident == "OwnedChildren" {
                    let it = inner_type(type_path);
                    match &it {
                        Type::Path(..) => {
                            children_fields.push(field_ident.clone());
                            children_types.push(it.clone());
                        }
                        _ => {}
                    }
                } else if type_ident == "OwnedParents" {
                    let it = inner_type(type_path);
                    match &it {
                        Type::Path(..) => {
                            parents_fields.push(field_ident.clone());
                            parents_types.push(it.clone());
                        }
                        _ => {}
                    }
                } else if type_ident == "OwnedChild" {
                    child_fields.push(field_ident.clone());
                    child_types.push(inner_type(type_path));
                } else if type_ident == "OwnedParent" {
                    parent_fields.push(field_ident.clone());
                    parent_types.push(inner_type(type_path));
                } else if VarName::VARIANTS.contains(&field_str.as_str())
                    && (VarValue::VARIANTS.contains(&type_ident.as_str())
                        || type_ident == "HexColor")
                {
                    var_fields.push(field_ident.clone());
                    var_types.push(ty.clone());
                } else {
                    data_fields.push(field_ident.clone());
                    data_types.push(ty.clone());
                }
                if !type_ident.starts_with("Owned")
                    && type_ident != "NodePart"
                    && type_ident != "NodeParts"
                {
                    all_data_fields.push(field_ident);
                    all_data_types.push(ty.clone());
                }
            }
            _ => unimplemented!(),
        }
    }
    pnf
}

pub fn strings_conversions(
    children_fields: &Vec<Ident>,
    children_types: &Vec<Type>,
    parents_fields: &Vec<Ident>,
    parents_types: &Vec<Type>,
    child_fields: &Vec<Ident>,
    child_types: &Vec<Type>,
    parent_fields: &Vec<Ident>,
    parent_types: &Vec<Type>,
) -> TokenStream {
    let shared_unpack = shared_unpack_id(
        children_fields,
        children_types,
        parents_fields,
        parents_types,
        child_fields,
        child_types,
        parent_fields,
        parent_types,
    );
    quote! {
        fn pack_fill(&self, pn: &mut PackedNodes) {
            let kind = self.kind().to_string();
            pn.add_node(kind.clone(), self.get_data(), self.id);
            #(
                if let Some(n) = self.#parent_fields.get_data() {
                    n.pack_fill(pn);
                    pn.link_parent_child(n.id, self.id, n.kind().to_string(), kind.clone());
                }
            )*
            #(
                if let Some(n) = self.#child_fields.get_data() {
                    n.pack_fill(pn);
                    pn.link_parent_child(self.id, n.id, kind.clone(), n.kind().to_string());
                }
            )*
            #(
                if let Some(parents_data) = self.#parents_fields.get_data() {
                    for n in parents_data {
                        n.pack_fill(pn);
                        pn.link_parent_child(n.id, self.id, n.kind().to_string(), kind.clone());
                    }
                }
            )*
            #(
                if let Some(children_data) = self.#children_fields.get_data() {
                    for n in children_data {
                        n.pack_fill(pn);
                        pn.link_parent_child(self.id, n.id, kind.clone(), n.kind().to_string());
                    }
                }
            )*
        }
        fn pack(&self) -> PackedNodes {
            let mut pn = PackedNodes::default();
            pn.root = self.id;
            self.pack_fill(&mut pn);
            pn
        }
        #shared_unpack
        fn reassign_ids(&mut self, next_id: &mut u64) {
            self.set_id(*next_id);
            *next_id += 1;
            #(
                if let Some(d) = self.#parent_fields.get_data_mut() {
                    d.reassign_ids(next_id);
                }
            )*
            #(
                if let Some(d) = self.#child_fields.get_data_mut() {
                    d.reassign_ids(next_id);
                }
            )*
            #(
                if let Some(parents_data) = self.#parents_fields.get_data_mut() {
                    for d in parents_data {
                        d.reassign_ids(next_id);
                    }
                }
            )*
            #(
                if let Some(children_data) = self.#children_fields.get_data_mut() {
                    for d in children_data {
                        d.reassign_ids(next_id);
                    }
                }
            )*
        }
    }
}

pub fn common_node_fns(
    struct_ident: &Ident,
    all_data_fields: &Vec<Ident>,
    all_data_types: &Vec<Type>,
) -> TokenStream {
    let fields_len = all_data_fields.len();
    quote! {
        #[allow(unused)]
        #[allow(dead_code)]
        #[allow(unused_mut)]
        impl #struct_ident {
            fn inject_data(&mut self, data: &str) -> Result<(), ExpressionError> {
                match ron::from_str::<Self>(data) {
                    Ok(v) => {
                        #(
                            self.#all_data_fields = v.#all_data_fields;
                        )*
                        Ok(())
                    }
                    Err(e) => Err(format!("Deserialize error: {e}").into()),
                }
            }
            fn from_data_fields(#(#all_data_fields: #all_data_types),*) -> Self {
                Self {
                    #(
                        #all_data_fields,
                    )*
                    ..default()
                }
            }
        }
        #[allow(unused)]
        #[allow(dead_code)]
        #[allow(unused_mut)]
        impl Serialize for #struct_ident {
            fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                let mut tup = serializer.serialize_tuple(#fields_len)?;
                #(
                    tup.serialize_element(&self.#all_data_fields)?;
                )*
                tup.end()
            }
        }
        #[allow(unused)]
        #[allow(dead_code)]
        #[allow(unused_mut)]
        impl<'de> Deserialize<'de> for #struct_ident {
            fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct TupleVisitor;
                impl<'de> Visitor<'de> for TupleVisitor {
                    type Value = (#(#all_data_types,)*);

                    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                        formatter.write_str("type err")
                    }
                    fn visit_seq<A>(self, mut seq: A) -> std::result::Result<Self::Value, A::Error>
                    where
                        A: de::SeqAccess<'de>,
                    {
                        Ok((
                            #(
                                seq.next_element::<#all_data_types>()?
                                    .ok_or_else(|| de::Error::invalid_length(0, &self))?,
                            )*
                        ))
                    }
                }
                let (#(#all_data_fields,)*) = deserializer.deserialize_tuple(#fields_len, TupleVisitor)?;
                Ok(Self::from_data_fields(#(#all_data_fields),*))
            }
        }
    }
}
pub fn shared_new_functions(
    all_data_fields: &Vec<Ident>,
    all_data_types: &Vec<Type>,
    one_fields: &Vec<Ident>,
    one_types: &Vec<Type>,
    many_fields: &Vec<Ident>,
    many_types: &Vec<Type>,
) -> TokenStream {
    quote! {
        pub fn new(
            owner: u64,
            #(
                #all_data_fields: #all_data_types,
            )*
        ) -> Self {
            Self {
                id: 0,
                owner,
                #(
                    #all_data_fields,
                )*
                ..Default::default()
            }
        }
        pub fn new_full(
            owner: u64,
            #(
                #all_data_fields: #all_data_types,
            )*
            #(
                #one_fields: #one_types,
            )*
            #(
                #many_fields: Vec<#many_types>,
            )*
        ) -> Self {
            Self {
                id: 0,
                owner,
                #(
                    #all_data_fields,
                )*
                #(
                    #one_fields: crate::NodePart::with_node(#one_fields),
                )*
                #(
                    #many_fields: crate::NodeParts::with_nodes(#many_fields),
                )*
                ..default()
            }
        }
    }
}

pub fn common_node_trait_fns(
    owned_children_types: &Vec<Type>,
    owned_parents_types: &Vec<Type>,
    owned_child_types: &Vec<Type>,
    owned_parent_types: &Vec<Type>,
) -> TokenStream {
    quote! {
        fn all_linked_parents() -> HashSet<NodeKind> {
            [
                #(
                    NodeKind::#owned_parents_types,
                )*
                #(
                    NodeKind::#owned_parent_types,
                )*
            ].into()
        }
        fn all_linked_children() -> HashSet<NodeKind> {
            [
                #(
                    NodeKind::#owned_children_types,
                )*
                #(
                    NodeKind::#owned_child_types,
                )*
            ].into()
        }
    }
}

pub fn shared_unpack_id(
    children_fields: &Vec<Ident>,
    children_types: &Vec<Type>,
    parents_fields: &Vec<Ident>,
    parents_types: &Vec<Type>,
    child_fields: &Vec<Ident>,
    child_types: &Vec<Type>,
    parent_fields: &Vec<Ident>,
    parent_types: &Vec<Type>,
) -> TokenStream {
    quote! {
        fn unpack_id(id: u64, pn: &PackedNodes) -> Option<Self> {
            let NodeData { kind, data } = pn.get(id)?;
            if !Self::kind_s().to_string().eq(kind) {
                panic!(
                    "Wrong node#{id} kind, expected {} got {}",
                    Self::kind_s(),
                    kind
                );
            }
            let mut d = Self::default();
            d.id = id;
            if let Err(e) = d.inject_data(data) {
                panic!("Unpack deserialize from data err: {e} data: {data}");
            }
            #(
                if let Some(child_id) = pn
                    .kind_children(id, NodeKind::#child_types.as_ref())
                    .get(0) {
                    if let Some(child_data) = #child_types::unpack_id(*child_id, pn) {
                        d.#child_fields.set_data(child_data);
                    }
                }
            )*
            #(
                if let Some(parent_id) = pn
                    .kind_parents(id, NodeKind::#parent_types.as_ref())
                    .get(0) {
                    if let Some(parent_data) = #parent_types::unpack_id(*parent_id, pn) {
                        d.#parent_fields.set_data(parent_data);
                    }
                }
            )*
            #(
                let parent_ids = pn
                    .kind_parents(id, NodeKind::#parents_types.as_ref());
                let parents_data: Vec<#parents_types> = parent_ids
                    .iter()
                    .filter_map(|parent_id| #parents_types::unpack_id(*parent_id, pn))
                    .collect();
                if !parents_data.is_empty() {
                    d.#parents_fields.set_data(parents_data);
                }
            )*
            #(
                let child_ids = pn
                    .kind_children(id, NodeKind::#children_types.as_ref());
                let children_data: Vec<#children_types> = child_ids
                    .iter()
                    .filter_map(|child_id| #children_types::unpack_id(*child_id, pn))
                    .collect();
                if !children_data.is_empty() {
                    d.#children_fields.set_data(children_data);
                }
            )*
            Some(d)
        }
    }
}
pub fn get_named_node_field(item_struct: &ItemStruct) -> Option<Ident> {
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

    Some(string_fields.remove(0))
}
pub fn parse_node_file(syntax_tree: File) -> (Vec<ItemStruct>, Vec<Ident>, Vec<Ident>) {
    let mut structs = Vec::new();
    let mut names: Vec<_> = Vec::new();
    let mut named_nodes: Vec<_> = Vec::new();
    for item in syntax_tree.items {
        if let syn::Item::Struct(mut item_struct) = item {
            let struct_name = item_struct.ident.clone();
            names.push(struct_name.clone());

            for attr in &item_struct.attrs {
                if attr.path().is_ident("node") {
                    let arg = attr.parse_args::<Ident>().unwrap();
                    if arg == "named" && get_named_node_field(&item_struct).is_some() {
                        named_nodes.push(struct_name.clone());
                    }
                    break;
                }
            }
            item_struct.attrs.clear();
            for field in item_struct.fields.iter_mut() {
                field.attrs.clear();
            }

            structs.push(item_struct);
        }
    }
    (structs, names, named_nodes)
}
