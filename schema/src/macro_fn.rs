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
