use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Meta};

#[proc_macro_derive(Node, attributes(var, content, named, many_to_one))]
pub fn derive_node(_input: TokenStream) -> TokenStream {
    quote! {}.into()
}

#[proc_macro_derive(Settings, attributes(setting))]
pub fn derive_settings(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("Settings derive only supports structs with named fields"),
        },
        _ => panic!("Settings derive only supports structs"),
    };

    let mut ui_code = Vec::new();

    for field in fields {
        let field_name = field.ident.as_ref().unwrap();
        let field_name_str = field_name.to_string();

        let mut label = field_name_str.clone();
        let mut ui_widget = None;
        let mut has_setting_attr = false;

        for attr in &field.attrs {
            if attr.path().is_ident("setting") {
                has_setting_attr = true;
                match &attr.meta {
                    Meta::List(list) => {
                        let tokens = &list.tokens;
                        let tokens_str = tokens.to_string();

                        if tokens_str.starts_with("slider") {
                            // Try to parse the entire tokens as a function call
                            let slider_args = extract_slider_from_tokens(&tokens_str);
                            if let Some((min_val, max_val)) = slider_args {
                                ui_widget = Some(quote! {
                                    if Slider::new(#label).ui(&mut settings.#field_name, #min_val..=#max_val, ui) {
                                        pd_mut(|d| d.client_settings.#field_name = settings.#field_name);
                                    }
                                });
                            } else {
                            }
                        } else if tokens_str.starts_with("checkbox") {
                            ui_widget = Some(quote! {
                                ui.label(#label);
                                if ui.checkbox(&mut settings.#field_name, "enabled").changed() {
                                    pd_mut(|d| d.client_settings.#field_name = settings.#field_name);
                                }
                            });
                        } else if tokens_str.starts_with("selector") {
                            if let Some(fn_name) = extract_function_name(&tokens_str) {
                                let fn_ident =
                                    syn::Ident::new(&fn_name, proc_macro2::Span::call_site());
                                ui_widget = Some(quote! {
                                    ui.label(#label);
                                    let options = #fn_ident();
                                    let (changed, _response) = Selector::ui_iter(&mut settings.#field_name, &options, ui);
                                    if changed {
                                        pd_mut(|d| d.client_settings.#field_name = settings.#field_name);
                                    }
                                });
                            } else {
                            }
                        } else if tokens_str.starts_with("enum") {
                            ui_widget = Some(quote! {
                                ui.label(#label);
                                let (old_value, _response) = Selector::ui_enum(&mut settings.#field_name, ui);
                                if old_value.is_some() {
                                    pd_mut(|d| d.client_settings.#field_name = settings.#field_name.clone());
                                }
                            });
                        } else if tokens_str.starts_with("edit") {
                            ui_widget = Some(quote! {
                                ui.label(#label);
                                ui.collapsing("edit", |ui| {
                                    if settings.#field_name.edit(ui).changed() {
                                        pd_mut(|d| d.client_settings.#field_name = settings.#field_name.clone());
                                    }
                                });
                            });
                        }

                        if let Some(label_str) = extract_label(&tokens_str) {
                            label = label_str;
                        }
                    }
                    _ => {}
                }
            }
        }

        if has_setting_attr {
            if let Some(widget) = ui_widget {
                ui_code.push(widget);
            } else {
                ui_code.push(quote! {
                    ui.label(#label);
                    if settings.#field_name.edit(ui) {
                        pd_mut(|d| d.client_settings.#field_name = settings.#field_name.clone());
                    }
                });
            }
            ui_code.push(quote! { ui.end_row(); });
        }
    }

    let expanded = quote! {
        impl #name {
            pub fn generate_settings_ui(&mut self, ui: &mut egui::Ui) {
                use crate::prelude::*;
                let settings = self;
                egui::Grid::new("attributed_settings_grid")
                    .num_columns(2)
                    .spacing([40.0, 4.0])
                    .show(ui, |ui| {
                        #(#ui_code)*
                    });
            }
        }
    };

    TokenStream::from(expanded)
}

fn extract_label(tokens_str: &str) -> Option<String> {
    if let Some(start) = tokens_str.find('"') {
        if let Some(end) = tokens_str[start + 1..].find('"') {
            return Some(tokens_str[start + 1..start + 1 + end].to_string());
        }
    }
    None
}

fn extract_slider_from_tokens(tokens_str: &str) -> Option<(f32, f32)> {
    // First split by comma to separate function call from label
    let main_parts: Vec<&str> = tokens_str.split(", \"").collect();
    if !main_parts.is_empty() {
        let func_part = main_parts[0].trim();
        if func_part.starts_with("slider(") && func_part.ends_with(")") {
            let inner = &func_part[7..func_part.len() - 1];
            let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
            if parts.len() == 3 {
                if let (Ok(_default), Ok(min), Ok(max)) = (
                    parts[0].parse::<f32>(),
                    parts[1].parse::<f32>(),
                    parts[2].parse::<f32>(),
                ) {
                    return Some((min, max));
                }
            }
        }
    }
    None
}

fn extract_function_name(tokens_str: &str) -> Option<String> {
    // First split by comma to separate function call from label
    let parts: Vec<&str> = tokens_str.split(',').collect();
    if !parts.is_empty() {
        let func_part = parts[0].trim();
        if func_part.starts_with("selector(") && func_part.ends_with(")") {
            let inner = &func_part[9..func_part.len() - 1];
            Some(inner.trim().to_string())
        } else {
            None
        }
    } else {
        None
    }
}
