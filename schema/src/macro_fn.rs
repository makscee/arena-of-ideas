use itertools::Itertools;
use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::ToTokens;
use quote::quote;
use syn::{Fields, GenericArgument, Ident, PathArguments, Type, TypePath};

#[derive(Default)]
pub struct ParsedNodeFields {
    pub one_fields: Vec<Ident>,
    pub one_fields_str: Vec<String>,
    pub one_types: Vec<proc_macro2::TokenStream>,
    pub many_fields: Vec<Ident>,
    pub many_fields_str: Vec<String>,
    pub many_types: Vec<proc_macro2::TokenStream>,
    pub var_fields: Vec<Ident>,
    pub var_types: Vec<Type>,
    pub data_fields: Vec<Ident>,
    pub data_fields_str: Vec<String>,
    pub data_types: Vec<Type>,
    pub data_type_ident: proc_macro2::TokenStream,
    pub all_data_fields: Vec<Ident>,
    pub all_data_types: Vec<Type>,
    pub linked_children_fields: Vec<Ident>,
    pub linked_children_types: Vec<Type>,
    pub linked_parents_fields: Vec<Ident>,
    pub linked_parents_types: Vec<Type>,
    pub linked_child_fields: Vec<Ident>,
    pub linked_child_types: Vec<Type>,
    pub linked_parent_fields: Vec<Ident>,
    pub linked_parent_types: Vec<Type>,
}

pub fn parse_node_fields(fields: &Fields) -> ParsedNodeFields {
    let mut pnf = ParsedNodeFields::default();
    let ParsedNodeFields {
        one_fields,
        one_fields_str,
        one_types,
        many_fields,
        many_fields_str,
        many_types,
        var_fields,
        var_types,
        data_fields,
        data_fields_str,
        data_types,
        data_type_ident,
        all_data_fields,
        all_data_types,
        linked_children_fields,
        linked_children_types,
        linked_parents_fields,
        linked_parents_types,
        linked_child_fields,
        linked_child_types,
        linked_parent_fields,
        linked_parent_types,
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
        match ty {
            syn::Type::Path(type_path) => {
                let type_ident = &type_path.path.segments.first().unwrap().ident;
                if type_ident == "OwnedChildren" || type_ident == "OwnedParents" {
                    let it = inner_type(type_path);
                    match &it {
                        Type::Path(..) => {
                            many_fields_str.push(field_ident.to_string());
                            many_fields.push(field_ident);
                            many_types.push(it.to_token_stream());
                        }
                        _ => {}
                    }
                } else if type_ident == "OwnedChild" || type_ident == "OwnedParent" {
                    one_fields_str.push(field_ident.to_string());
                    one_fields.push(field_ident);
                    one_types.push(inner_type(type_path).to_token_stream());
                } else if type_ident == "LinkedChild" {
                    linked_child_fields.push(field_ident.clone());
                    linked_child_types.push(inner_type(type_path));
                    all_data_fields.push(field_ident);
                    all_data_types.push(ty.clone());
                } else if type_ident == "LinkedChildren" {
                    linked_children_fields.push(field_ident.clone());
                    linked_children_types.push(inner_type(type_path));
                    all_data_fields.push(field_ident);
                    all_data_types.push(ty.clone());
                } else if type_ident == "LinkedParent" {
                    linked_parent_fields.push(field_ident.clone());
                    linked_parent_types.push(inner_type(type_path));
                    all_data_fields.push(field_ident);
                    all_data_types.push(ty.clone());
                } else if type_ident == "LinkedParents" {
                    linked_parents_fields.push(field_ident.clone());
                    linked_parents_types.push(inner_type(type_path));
                    all_data_fields.push(field_ident);
                    all_data_types.push(ty.clone());
                } else if type_ident == "i32"
                    || type_ident == "f32"
                    || type_ident == "String"
                    || type_ident == "color"
                    || type_ident == "HexColor"
                {
                    var_fields.push(field_ident.clone());
                    var_types.push(ty.clone());
                } else {
                    data_fields_str.push(field_ident.to_string());
                    data_fields.push(field_ident);
                    data_types.push(ty.clone());
                }
            }
            _ => unimplemented!(),
        }
    }
    all_data_fields.append(&mut var_fields.clone());
    all_data_fields.append(&mut data_fields.clone());

    all_data_types.append(&mut var_types.clone());
    all_data_types.append(&mut data_types.clone());

    *data_type_ident = quote! { (#(#all_data_types),*) };
    pnf
}

pub fn strings_conversions(
    one_fields: &Vec<Ident>,
    _component_fields_str: &Vec<String>,
    one_types: &Vec<TokenStream>,
    many_fields: &Vec<Ident>,
    _child_fields_str: &Vec<String>,
    many_types: &Vec<TokenStream>,
    linked_children_fields: &Vec<Ident>,
    linked_children_types: &Vec<Type>,
    linked_parents_fields: &Vec<Ident>,
    linked_parents_types: &Vec<Type>,
    linked_child_fields: &Vec<Ident>,
    linked_child_types: &Vec<Type>,
    linked_parent_fields: &Vec<Ident>,
    linked_parent_types: &Vec<Type>,
) -> TokenStream {
    let shared_unpack = shared_unpack_id_fix(
        one_fields,
        one_types,
        many_fields,
        many_types,
        linked_children_fields,
        linked_children_types,
        linked_parents_fields,
        linked_parents_types,
        linked_child_fields,
        linked_child_types,
        linked_parent_fields,
        linked_parent_types,
    );
    quote! {
        fn pack_fill(&self, pn: &mut PackedNodes) {
            let kind = self.kind().to_string();
            pn.add_node(kind.clone(), self.get_data(), self.id);
            #(
                if let Some(n) = self.#one_fields.as_ref() {
                    n.pack_fill(pn);
                    pn.link_parent_child(n.id, self.id, n.kind().to_string(), kind.clone());
                }
            )*
            #(
                for n in &self.#many_fields {
                    n.pack_fill(pn);
                    pn.link_parent_child(self.id, n.id, kind.clone(), n.kind().to_string());
                }
            )*
            #(
                if let Some(parent) = &self.linked_parent_fields {
                    pn.link_parent_child(*parent, self.id, NodeKind::#linked_parent_types.to_string(), kind.clone());
                }
            )*
            #(
                if let Some(child) = &self.linked_child_fields {
                    pn.link_parent_child(self.id, *child, NodeKind::#linked_child_types.to_string(), kind.clone());
                }
            )*
            #(
                for child in &self.#linked_children_fields.ids {
                    pn.link_parent_child(self.id, *child, kind.clone(), NodeKind::#linked_children_types.to_string());
                }
            )*
            #(
                for parent in &self.#linked_parents_fields.ids {
                    pn.link_parent_child(*parent, self.id, NodeKind::#linked_parents_types.to_string(), kind.clone());
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
            let id = self.id();
            #(
                if let Some(d) = self.#one_fields.as_mut() {
                    d.reassign_ids(next_id);
                }
            )*
            #(
                for d in &mut self.#many_fields {
                    d.reassign_ids(next_id);
                }
            )*
        }
    }
}

pub fn common_node_fns(
    struct_ident: &Ident,
    all_data_fields: &Vec<Ident>,
    all_data_types: &Vec<Type>,
    one_fields: &Vec<Ident>,
    one_types: &Vec<TokenStream>,
) -> TokenStream {
    let component_link_fields_mut = one_fields
        .iter()
        .map(|i| Ident::new(&format!("{i}_mut"), Span::call_site()))
        .collect_vec();
    let component_link_fields_err = one_fields
        .iter()
        .map(|i| format!("Failed to get field {i}").leak())
        .collect_vec();
    let fields_len = all_data_fields.len();
    quote! {
        #[allow(unused)]
        #[allow(dead_code)]
        #[allow(unused_mut)]
        impl #struct_ident {
            #(
                pub fn #one_fields(&self) -> &#one_types {
                    self.#one_fields.as_ref().expect(#component_link_fields_err)
                }
            )*
            #(
                pub fn #component_link_fields_mut<'a>(&'a mut self) -> &'a mut #one_types {
                    self.#one_fields.as_mut().expect(#component_link_fields_err)
                }
            )*
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
    _struct_ident: &Ident,
    all_data_fields: &Vec<Ident>,
    all_data_types: &Vec<Type>,
    one_fields: &Vec<Ident>,
    one_types: &Vec<TokenStream>,
    many_fields: &Vec<Ident>,
    many_types: &Vec<TokenStream>,
    _is_server: bool,
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
                    #one_fields: Some(#one_fields),
                )*
                #(
                    #many_fields,
                )*
                ..default()
            }
        }
    }
}

pub fn common_node_trait_fns(
    _struct_ident: &Ident,
    one_types: &Vec<TokenStream>,
    many_types: &Vec<TokenStream>,
    linked_children_types: &Vec<Type>,
    linked_parents_types: &Vec<Type>,
) -> TokenStream {
    quote! {
        fn owned_kinds() -> HashSet<NodeKind> {
            let mut kinds = Self::owned_parents();
            kinds.extend(Self::owned_children());
            kinds
        }
        fn owned_parents() -> HashSet<NodeKind> {
            [
                #(
                    NodeKind::#one_types,
                )*
            ].into()
        }
        fn owned_children() -> HashSet<NodeKind> {
            [
                #(
                    NodeKind::#many_types,
                )*
            ].into()
        }
        fn linked_children() -> HashSet<NodeKind> {
            [
                #(
                    NodeKind::#linked_children_types,
                )*
            ].into()
        }
        fn linked_parents() -> HashSet<NodeKind> {
            [
                #(
                    NodeKind::#linked_parents_types,
                )*
            ].into()
        }
        fn component_kinds() -> HashSet<NodeKind> {
            [
                #(
                    NodeKind::#one_types,
                )*
            ].into()
        }
        fn children_kinds() -> HashSet<NodeKind> {
            [
                #(
                    NodeKind::#many_types,
                )*
            ].into()
        }
    }
}

pub fn shared_unpack_id_fix(
    one_fields: &Vec<Ident>,
    one_types: &Vec<TokenStream>,
    many_fields: &Vec<Ident>,
    many_types: &Vec<TokenStream>,
    linked_children_fields: &Vec<Ident>,
    linked_children_types: &Vec<Type>,
    linked_parents_fields: &Vec<Ident>,
    linked_parents_types: &Vec<Type>,
    linked_child_fields: &Vec<Ident>,
    linked_child_types: &Vec<Type>,
    linked_parent_fields: &Vec<Ident>,
    linked_parent_types: &Vec<Type>,
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
            d.id = if id == 0 {
                // Will be assigned proper ID when unpacked to context
                0
            } else {
                id
            };
            if let Err(e) = d.inject_data(data) {
                panic!("Unpack deserialize from data err: {e} data: {data}");
            }
            #(
                d.#one_fields = pn
                    .kind_parents(id, NodeKind::#one_types.as_ref())
                    .get(0)
                    .and_then(|id| #one_types::unpack_id(*id, pn));
            )*
            #(
                d.#many_fields = pn
                    .kind_children(id, NodeKind::#many_types.as_ref())
                    .into_iter()
                    .filter_map(|id| #many_types::unpack_id(id, pn))
                    .collect();
            )*
            #(
                d.#linked_children_fields = linked_children(pn.kind_children(id, NodeKind::#linked_children_types.as_ref()));
            )*
            #(
                d.#linked_parents_fields = linked_parents(pn.kind_parents(id, NodeKind::#linked_parents_types.as_ref()));
            )*
            #(
                d.#linked_child_fields = linked_parent(pn.kind_children(id, NodeKind::#linked_child_types.as_ref()).first());
            )*
            #(
                d.#linked_parent_fields = linked_parent(pn.kind_parents(id, NodeKind::#linked_parent_types.as_ref()).first());
            )*
            Some(d)
        }
    }
}
