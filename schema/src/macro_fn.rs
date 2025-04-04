use itertools::Itertools;
use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;
use syn::{Fields, GenericArgument, Ident, PathArguments, Type, TypePath};

pub struct ParsedNodeFields {
    pub component_fields: Vec<Ident>,
    pub component_fields_str: Vec<String>,
    pub component_types: Vec<proc_macro2::TokenStream>,
    pub child_fields: Vec<Ident>,
    pub child_fields_str: Vec<String>,
    pub child_types: Vec<proc_macro2::TokenStream>,
    pub var_fields: Vec<Ident>,
    pub var_types: Vec<Type>,
    pub data_fields: Vec<Ident>,
    pub data_fields_str: Vec<String>,
    pub data_types: Vec<Type>,
    pub data_type_ident: proc_macro2::TokenStream,
    pub all_data_fields: Vec<Ident>,
    pub all_data_types: Vec<Type>,
}

pub fn parse_node_fields(fields: &Fields) -> ParsedNodeFields {
    let mut component_fields = Vec::default();
    let mut component_fields_str = Vec::default();
    let mut component_types: Vec<proc_macro2::TokenStream> = Vec::default();
    let mut child_fields = Vec::default();
    let mut child_fields_str = Vec::default();
    let mut child_types: Vec<proc_macro2::TokenStream> = Vec::default();
    let mut var_fields = Vec::default();
    let mut var_types = Vec::default();
    let mut data_fields = Vec::default();
    let mut data_fields_str = Vec::default();
    let mut data_types = Vec::default();
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
                if type_ident == "NodeChildren" {
                    let it = inner_type(type_path);
                    match &it {
                        Type::Path(..) => {
                            child_fields_str.push(field_ident.to_string());
                            child_fields.push(field_ident);
                            child_types.push(it.to_token_stream());
                        }
                        _ => {}
                    }
                } else if type_ident == "NodeComponent" {
                    component_fields_str.push(field_ident.to_string());
                    component_fields.push(field_ident);
                    component_types.push(inner_type(type_path).to_token_stream());
                } else if type_ident == "i32"
                    || type_ident == "f32"
                    || type_ident == "String"
                    || type_ident == "Color"
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
    let mut all_data_fields = var_fields.clone();
    all_data_fields.append(&mut data_fields.clone());
    let mut all_data_types = var_types.clone();
    all_data_types.append(&mut data_types.clone());

    let data_type_ident = quote! { (#(#all_data_types),*) };
    ParsedNodeFields {
        component_fields,
        component_fields_str,
        component_types,
        child_fields,
        child_fields_str,
        child_types,
        var_fields,
        var_types,
        data_fields,
        data_fields_str,
        data_types,
        data_type_ident,
        all_data_fields,
        all_data_types,
    }
}

pub fn strings_conversions(
    component_fields: &Vec<Ident>,
    _component_fields_str: &Vec<String>,
    component_types: &Vec<TokenStream>,
    child_fields: &Vec<Ident>,
    _child_fields_str: &Vec<String>,
    child_types: &Vec<TokenStream>,
) -> TokenStream {
    quote! {
        fn to_tnodes(&self) -> Vec<TNode> {
            let mut v = [self.to_tnode()].to_vec();
            #(
                if let Some(d) = self.#component_fields.as_ref() {
                    v.extend(d.to_tnodes());
                }
            )*
            #(
                for d in &self.#child_fields {
                    v.extend(d.to_tnodes());
                }
            )*
            v
        }
        fn from_tnodes(id: u64, nodes: &Vec<TNode>) -> Option<Self> {
            let mut node = nodes
                .into_iter()
                .find(|n| n.id == id)?
                .to_node::<Self>()
                .ok()?;
            #(
                let kind = NodeKind::#component_types.to_string();
                node.#component_fields = nodes
                    .into_iter()
                    .find(|n| n.parent == id && n.kind == kind)
                    .and_then(|n| #component_types::from_tnodes(n.id, nodes));
            )*
            #(
                let kind = NodeKind::#child_types.to_string();
                node.#child_fields = nodes
                    .into_iter()
                    .filter_map(|n| {
                        if n.parent == id && n.kind == kind {
                            #child_types::from_tnodes(n.id, nodes)
                        } else {
                            None
                        }
                    })
                    .collect();
            )*

            Some(node)
        }
        fn reassign_ids(&mut self, next_id: &mut u64) {
            self.set_id(*next_id);
            *next_id += 1;
            let id = self.id();
            #(
                if let Some(d) = self.#component_fields.as_mut() {
                    d.set_parent(id);
                    d.reassign_ids(next_id);
                }
            )*
            #(
                for d in &mut self.#child_fields {
                    d.set_parent(id);
                    d.reassign_ids(next_id);
                }
            )*
        }
    }
}

pub fn table_conversions(
    component_fields: &Vec<Ident>,
    component_types: &Vec<TokenStream>,
    child_fields: &Vec<Ident>,
    child_types: &Vec<TokenStream>,
) -> TokenStream {
    quote! {
        fn with_components(&mut self, ctx: &ReducerContext) -> &mut Self {
            #(
                self.#component_fields = self.find_child::<#component_types>(ctx).ok()
                    .map(|mut d| std::mem::take(d.with_components(ctx)
                        .with_children(ctx))
                    );
            )*
            self
        }
        fn with_children(&mut self, ctx: &ReducerContext) -> &mut Self {
            #(
                self.#child_fields = self.collect_children::<#child_types>(ctx)
                    .into_iter()
                    .map(|mut n| std::mem::take(n.with_components(ctx).with_children(ctx)))
                    .collect();
            )*
            self
        }
        fn save(mut self, ctx: &ReducerContext) {
            self.update_self(ctx);
            #(
                if let Some(mut d) = self.#component_fields.take() {
                    d.save(ctx);
                }
            )*
            #(
                for mut d in std::mem::take(&mut self.#child_fields) {
                    d.save(ctx);
                }
            )*
        }
    }
}
pub fn common_node_fns(
    struct_ident: &Ident,
    _all_data_fields: &Vec<Ident>,
    _all_data_types: &Vec<Type>,
    component_fields: &Vec<Ident>,
    component_types: &Vec<TokenStream>,
) -> TokenStream {
    let component_link_fields_mut = component_fields
        .iter()
        .map(|i| Ident::new(&format!("{i}_mut"), Span::call_site()))
        .collect_vec();
    let component_link_fields_err = component_fields
        .iter()
        .map(|i| format!("Failed to get field {i}").leak())
        .collect_vec();
    quote! {
        impl #struct_ident {
            #(
                pub fn #component_fields(&self) -> &#component_types {
                    self.#component_fields.as_ref().expect(#component_link_fields_err)
                }
            )*
            #(
                pub fn #component_link_fields_mut<'a>(&'a mut self) -> &'a mut #component_types {
                    self.#component_fields.as_mut().expect(#component_link_fields_err)
                }
            )*
        }
    }
}
pub fn common_node_trait_fns(
    _struct_ident: &Ident,
    component_types: &Vec<TokenStream>,
    child_types: &Vec<TokenStream>,
) -> TokenStream {
    quote! {
        fn component_kinds() -> HashSet<NodeKind> {
            [
                #(
                    NodeKind::#component_types,
                )*
            ].into()
        }
        fn children_kinds() -> HashSet<NodeKind> {
            [
                #(
                    NodeKind::#child_types,
                )*
            ].into()
        }
    }
}
