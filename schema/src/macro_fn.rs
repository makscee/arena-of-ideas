use itertools::Itertools;
use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;
use syn::{Fields, GenericArgument, Ident, PathArguments, Type, TypePath};

pub struct ParsedNodeFields {
    pub component_link_fields: Vec<Ident>,
    pub component_link_fields_str: Vec<String>,
    pub component_link_types: Vec<proc_macro2::TokenStream>,
    pub child_link_fields: Vec<Ident>,
    pub child_link_fields_str: Vec<String>,
    pub child_link_types: Vec<proc_macro2::TokenStream>,
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
    let mut component_link_fields = Vec::default();
    let mut component_link_fields_str = Vec::default();
    let mut component_link_types: Vec<proc_macro2::TokenStream> = Vec::default();
    let mut child_link_fields = Vec::default();
    let mut child_link_fields_str = Vec::default();
    let mut child_link_types: Vec<proc_macro2::TokenStream> = Vec::default();
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
                            child_link_fields_str.push(field_ident.to_string());
                            child_link_fields.push(field_ident);
                            child_link_types.push(it.to_token_stream());
                        }
                        _ => {}
                    }
                } else if type_ident == "NodeComponent" {
                    component_link_fields_str.push(field_ident.to_string());
                    component_link_fields.push(field_ident);
                    component_link_types.push(inner_type(type_path).to_token_stream());
                } else if type_ident == "i32"
                    || type_ident == "f32"
                    || type_ident == "String"
                    || type_ident == "Color"
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

    if all_data_types.is_empty() {
        panic!("No data fields found");
    }
    let data_type_ident = quote! { (#(#all_data_types),*) };
    ParsedNodeFields {
        component_link_fields,
        component_link_fields_str,
        component_link_types,
        child_link_fields,
        child_link_fields_str,
        child_link_types,
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
    component_link_fields: &Vec<Ident>,
    component_link_fields_str: &Vec<String>,
    component_link_types: &Vec<TokenStream>,
    child_link_fields: &Vec<Ident>,
    child_link_fields_str: &Vec<String>,
    child_link_types: &Vec<TokenStream>,
) -> TokenStream {
    quote! {
        fn to_strings(&self, parent: usize, field: &str, strings: &mut Vec<String>) {
            let entry = format!("{parent} {field} {}", self.get_data());
            let i = strings.len();
            strings.push(entry);
            #(
                if let Some(d) = &self.#component_link_fields {
                    d.to_strings(i, #component_link_fields_str, strings);
                }
            )*
            #(
                for d in &self.#child_link_fields {
                    d.to_strings(i, #child_link_fields_str, strings);
                }
            )*
        }
        fn from_strings(i: usize, strings: &Vec<String>) -> Option<Self> {
            if i >= strings.len() {
                return None;
            }
            let (_, _, data) = strings[i].splitn(3, ' ').collect_tuple()?;
            let mut d = Self::default();
            d.inject_data(data);
            let i_str = i.to_string();
            #(
                d.#component_link_fields = strings.iter().enumerate().skip(i).find_map(|(i, s)| {
                    let (parent, field, _) = s.splitn(3, ' ').collect_tuple()?;
                    if i_str.eq(parent) && field.eq(#component_link_fields_str) {
                        #component_link_types::from_strings(i, strings)
                    } else {
                        None
                    }
                });
            )*
            #(
                d.#child_link_fields = strings.iter().enumerate().skip(i).filter_map(|(i, s)| {
                    let (parent, field, _) = s.splitn(3, ' ').collect_tuple()?;
                    if i_str.eq(parent) && field.eq(#child_link_fields_str) {
                        #child_link_types::from_strings(i, strings)
                    } else {
                        None
                    }
                }).collect();
            )*
            Some(d)
        }
    }
}

pub fn table_conversions(
    component_link_fields: &Vec<Ident>,
    component_link_types: &Vec<TokenStream>,
    child_link_fields: &Vec<Ident>,
    child_link_types: &Vec<TokenStream>,
) -> TokenStream {
    quote! {
        fn with_components(mut self, ctx: &ReducerContext) -> Self {
            #(
                self.#component_link_fields = #component_link_types::get(ctx, self.id())
                    .map(|d| d.with_components(ctx)
                        .with_children(ctx)
                    );
            )*
            self
        }
        fn with_children(mut self, ctx: &ReducerContext) -> Self {
            let children = ctx
                .db
                .nodes_relations()
                .parent()
                .filter(self.id())
                .map(|r| r.id)
                .collect_vec();
            #(
                self.#child_link_fields = children
                    .iter()
                    .filter_map(|id| #child_link_types::get(ctx, *id).map(|d|
                        d.with_components(ctx).with_children(ctx)))
                    .collect();
            )*
            self
        }
        fn save(mut self, ctx: &ReducerContext) {
            if self.id.is_none() {
                self.id = Some(next_id(ctx));
            }
            let id = self.id();
            self.insert_self(ctx);
            #(
                if let Some(mut d) = self.#component_link_fields.take() {
                    d.id = Some(id);
                    d.save(ctx);
                }
            )*
            #(
                for d in std::mem::take(&mut self.#child_link_fields) {
                    d.set_parent(ctx, id);
                    d.save(ctx);
                }
            )*
        }
    }
}
pub fn common_node_fns(
    struct_ident: &Ident,
    all_data_fields: &Vec<Ident>,
    all_data_types: &Vec<Type>,
    component_link_fields: &Vec<Ident>,
    component_link_types: &Vec<TokenStream>,
) -> TokenStream {
    let component_link_fields_mut = component_link_fields
        .iter()
        .map(|i| Ident::new(&format!("{i}_mut"), Span::call_site()))
        .collect_vec();
    quote! {
        impl #struct_ident {
            pub fn new(
                #(
                    #all_data_fields: #all_data_types,
                )*
            ) -> Self {
                Self {
                    #(
                        #all_data_fields,
                    )*
                    ..default()
                }
            }
            pub fn new_full(
                #(
                    #all_data_fields: #all_data_types,
                )*
                #(
                    #component_link_fields: #component_link_types,
                )*
            ) -> Self {
                Self {
                    #(
                        #all_data_fields,
                    )*
                    #(
                        #component_link_fields: Some(#component_link_fields),
                    )*
                    ..default()
                }
            }
            #(
                pub fn #component_link_fields(&self) -> &#component_link_types {
                    self.#component_link_fields.as_ref().unwrap()
                }
            )*
            #(
                pub fn #component_link_fields_mut<'a>(&'a mut self) -> &'a mut #component_link_types {
                    self.#component_link_fields.as_mut().unwrap()
                }
            )*
        }
    }
}
