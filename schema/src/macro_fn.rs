use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;
use syn::{Fields, GenericArgument, Ident, PathArguments, Type, TypePath};

pub struct ParsedNodeFields {
    pub option_link_fields: Vec<Ident>,
    pub option_link_fields_str: Vec<String>,
    pub option_link_types: Vec<proc_macro2::TokenStream>,
    pub vec_link_fields: Vec<Ident>,
    pub vec_link_fields_str: Vec<String>,
    pub vec_link_types: Vec<proc_macro2::TokenStream>,
    pub vec_box_link_fields: Vec<Ident>,
    pub vec_box_link_fields_str: Vec<String>,
    pub vec_box_link_types: Vec<proc_macro2::TokenStream>,
    pub var_fields: Vec<Ident>,
    pub var_types: Vec<Type>,
    pub data_fields: Vec<Ident>,
    pub data_types: Vec<Type>,
    pub data_type_ident: proc_macro2::TokenStream,
    pub all_data_fields: Vec<Ident>,
    pub all_data_types: Vec<Type>,
}

pub fn parse_node_fields(fields: &Fields) -> ParsedNodeFields {
    let mut option_link_fields = Vec::default();
    let mut option_link_fields_str = Vec::default();
    let mut option_link_types: Vec<proc_macro2::TokenStream> = Vec::default();
    let mut vec_link_fields = Vec::default();
    let mut vec_link_fields_str = Vec::default();
    let mut vec_link_types: Vec<proc_macro2::TokenStream> = Vec::default();
    let mut vec_box_link_fields = Vec::default();
    let mut vec_box_link_fields_str = Vec::default();
    let mut vec_box_link_types: Vec<proc_macro2::TokenStream> = Vec::default();
    let mut var_fields = Vec::default();
    let mut var_types = Vec::default();
    let mut data_fields = Vec::default();
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
                if type_ident == "Vec" {
                    let it = inner_type(type_path);
                    match &it {
                        Type::Path(type_path) => {
                            if type_path
                                .path
                                .segments
                                .first()
                                .is_some_and(|t| t.ident == "Box")
                            {
                                let it = inner_type(&type_path);
                                vec_box_link_fields_str.push(field_ident.to_string());
                                vec_box_link_fields.push(field_ident);
                                vec_box_link_types.push(it.to_token_stream());
                            } else {
                                vec_link_fields_str.push(field_ident.to_string());
                                vec_link_fields.push(field_ident);
                                vec_link_types.push(it.to_token_stream());
                            }
                        }
                        _ => {}
                    }
                } else if type_ident == "Option" {
                    option_link_fields_str.push(field_ident.to_string());
                    option_link_fields.push(field_ident);
                    option_link_types.push(inner_type(type_path).to_token_stream());
                } else if type_ident == "i32"
                    || type_ident == "f32"
                    || type_ident == "String"
                    || type_ident == "Color"
                {
                    var_fields.push(field_ident.clone());
                    var_types.push(ty.clone());
                } else {
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
        option_link_fields,
        option_link_fields_str,
        option_link_types,
        vec_link_fields,
        vec_link_fields_str,
        vec_link_types,
        vec_box_link_fields,
        vec_box_link_fields_str,
        vec_box_link_types,
        var_fields,
        var_types,
        data_fields,
        data_types,
        data_type_ident,
        all_data_fields,
        all_data_types,
    }
}

pub fn strings_conversions(
    option_link_fields: &Vec<Ident>,
    option_link_fields_str: &Vec<String>,
    option_link_types: &Vec<TokenStream>,
    vec_link_fields: &Vec<Ident>,
    vec_link_fields_str: &Vec<String>,
    vec_link_types: &Vec<TokenStream>,
    vec_box_link_fields: &Vec<Ident>,
    vec_box_link_fields_str: &Vec<String>,
    vec_box_link_types: &Vec<TokenStream>,
) -> TokenStream {
    quote! {
        fn to_strings(&self, parent: usize, field: &str, strings: &mut Vec<String>) {
            let entry = format!("{parent} {field} {}", self.get_data());
            let i = strings.len();
            strings.push(entry);
            #(
                if let Some(d) = &self.#option_link_fields {
                    d.to_strings(i, #option_link_fields_str, strings);
                }
            )*
            #(
                for d in &self.#vec_link_fields {
                    d.to_strings(i, #vec_link_fields_str, strings);
                }
            )*
            #(
                for d in &self.#vec_box_link_fields {
                    d.to_strings(i, #vec_box_link_fields_str, strings);
                }
            )*
        }
        fn from_strings(i: usize, strings: &Vec<String>) -> Option<Self> {
            let (_, _, data) = strings[i].splitn(3, ' ').collect_tuple()?;
            let mut d = Self::default();
            d.inject_data(data);
            let i_str = i.to_string();
            #(
                d.#option_link_fields = strings.iter().enumerate().skip(i).find_map(|(i, s)| {
                    let (parent, field, _) = s.splitn(3, ' ').collect_tuple()?;
                    if i_str.eq(parent) && field.eq(#option_link_fields_str) {
                        #option_link_types::from_strings(i, strings)
                    } else {
                        None
                    }
                });
            )*
            #(
                d.#vec_link_fields = strings.iter().enumerate().skip(i).filter_map(|(i, s)| {
                    let (parent, field, _) = s.splitn(3, ' ').collect_tuple()?;
                    if i_str.eq(parent) && field.eq(#vec_link_fields_str) {
                        #vec_link_types::from_strings(i, strings)
                    } else {
                        None
                    }
                }).collect();
            )*
            #(
                d.#vec_box_link_fields = strings.iter().enumerate().skip(i).filter_map(|(i, s)| {
                    let (parent, field, _) = s.splitn(3, ' ').collect_tuple()?;
                    if i_str.eq(parent) && field.eq(#vec_box_link_fields_str) {
                        #vec_box_link_types::from_strings(i, strings).map(|v| Box::new(v))
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
    option_link_fields: &Vec<Ident>,
    option_link_types: &Vec<TokenStream>,
    vec_link_fields: &Vec<Ident>,
    vec_link_types: &Vec<TokenStream>,
    vec_box_link_fields: &Vec<Ident>,
    vec_box_link_types: &Vec<TokenStream>,
) -> TokenStream {
    quote! {
        fn from_table(ctx: &ReducerContext, id: u64) -> Option<Self> {
            let data = ctx
                .db
                .nodes_match()
                .key()
                .find(Self::kind_s().key(id))?
                .data;
            let mut d = Self::default();
            d.inject_data(&data);
            let children = ctx
                .db
                .nodes_relations()
                .parent()
                .filter(id)
                .map(|r| r.id)
                .collect_vec();
            #(
                d.#option_link_fields = #option_link_types::from_table(ctx, id);
            )*
            #(
                d.#vec_link_fields = children
                    .iter()
                    .filter_map(|id| #vec_link_types::from_table(ctx, *id))
                    .collect();
            )*
            #(
                d.#vec_box_link_fields = children
                    .iter()
                    .filter_map(|id| #vec_box_link_types::from_table(ctx, *id))
                    .map(|d| Box::new(d))
                    .collect();
            )*
            Some(d)
        }
    }
}
