use itertools::Itertools;
use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::ToTokens;
use quote::quote;
use syn::{Fields, GenericArgument, Ident, PathArguments, Type, TypePath};

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
    pub parent_fields: Vec<Ident>,
    pub parent_types: Vec<Type>,
}

pub fn parse_node_fields(fields: &Fields) -> ParsedNodeFields {
    let mut one_fields = Vec::default();
    let mut one_fields_str = Vec::default();
    let mut one_types: Vec<proc_macro2::TokenStream> = Vec::default();
    let mut many_fields = Vec::default();
    let mut many_fields_str = Vec::default();
    let mut many_types: Vec<proc_macro2::TokenStream> = Vec::default();
    let mut var_fields = Vec::default();
    let mut var_types = Vec::default();
    let mut data_fields = Vec::default();
    let mut data_fields_str = Vec::default();
    let mut data_types = Vec::default();
    let mut parent_fields = Vec::default();
    let mut parent_types = Vec::default();
    fn inner_type(type_path: &TypePath) -> Type {
        match &type_path.path.segments.first().unwrap().arguments {
            PathArguments::AngleBracketed(arg) => match arg.args.first().unwrap() {
                GenericArgument::Type(t) => t.clone(),
                _ => unimplemented!(),
            },
            _ => unimplemented!(),
        }
    }
    let mut all_data_fields: Vec<Ident> = Vec::default();
    let mut all_data_types: Vec<Type> = Vec::default();
    for field in fields.iter() {
        let ty = &field.ty;
        let field_ident = field.ident.clone().unwrap();
        match ty {
            syn::Type::Path(type_path) => {
                let type_ident = &type_path.path.segments.first().unwrap().ident;
                if type_ident == "ChildComponents" || type_ident == "ParentComponents" {
                    let it = inner_type(type_path);
                    match &it {
                        Type::Path(..) => {
                            many_fields_str.push(field_ident.to_string());
                            many_fields.push(field_ident);
                            many_types.push(it.to_token_stream());
                        }
                        _ => {}
                    }
                } else if type_ident == "ParentComponent" || type_ident == "ChildComponent" {
                    one_fields_str.push(field_ident.to_string());
                    one_fields.push(field_ident);
                    one_types.push(inner_type(type_path).to_token_stream());
                } else if type_ident == "ParentLinks" {
                    parent_fields.push(field_ident.clone());
                    parent_types.push(inner_type(type_path));
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

    let data_type_ident = quote! { (#(#all_data_types),*) };
    ParsedNodeFields {
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
        parent_fields,
        parent_types,
    }
}

pub fn strings_conversions(
    one_fields: &Vec<Ident>,
    _component_fields_str: &Vec<String>,
    one_types: &Vec<TokenStream>,
    many_fields: &Vec<Ident>,
    _child_fields_str: &Vec<String>,
    many_types: &Vec<TokenStream>,
    parent_fields: &Vec<Ident>,
    parent_types: &Vec<Type>,
) -> TokenStream {
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
                for parent in &self.#parent_fields.ids {
                    pn.link_parent_child(*parent, self.id, NodeKind::#parent_types.to_string(), kind.clone());
                }
            )*
        }
        fn pack(&self) -> PackedNodes {
            let mut pn = PackedNodes::default();
            pn.root = self.id;
            self.pack_fill(&mut pn);
            pn
        }
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
                d.#parent_fields = parent_links(pn.kind_parents(id, NodeKind::#parent_types.as_ref()));
            )*
            Some(d)
        }
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
pub fn common_node_trait_fns(
    _struct_ident: &Ident,
    one_types: &Vec<TokenStream>,
    many_types: &Vec<TokenStream>,
) -> TokenStream {
    quote! {
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
