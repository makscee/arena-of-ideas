use darling::FromMeta;
use itertools::Itertools;
use parse::Parser;
use proc_macro::TokenStream;
use punctuated::Punctuated;
use quote::ToTokens;
use syn::*;
#[macro_use]
extern crate quote;

#[proc_macro_attribute]
pub fn node(args: TokenStream, item: TokenStream) -> TokenStream {
    let a = parse_macro_input!(args with Punctuated<Path, Token![,]>::parse_terminated);
    let mut on_unpack = proc_macro2::TokenStream::new();
    for i in a.iter() {
        let i = i.get_ident().unwrap().to_string();
        match i.as_str() {
            "on_unpack" => {
                on_unpack = quote! { self.on_unpack(entity, commands); }.into_token_stream();
            }
            _ => unimplemented!(),
        }
    }
    let mut input = parse_macro_input!(item as syn::DeriveInput);
    let struct_ident = &input.ident;

    enum NodeType {
        Name,
        Data,
        OnlyData,
    }

    let result = match &mut input.data {
        syn::Data::Struct(syn::DataStruct {
            struct_token: _,
            fields,
            semi_token: _,
        }) => {
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
            let nt = if all_data_fields.contains(&Ident::from_string("name").unwrap()) {
                NodeType::Name
            } else if !option_link_fields.is_empty() || !vec_box_link_fields.is_empty() {
                NodeType::Data
            } else {
                NodeType::OnlyData
            };
            let data_from_dir = match nt {
                NodeType::Name => quote! {
                    let data = &format!("\"{}\"", dir.path().file_name()?.to_str()?);
                },
                NodeType::Data => quote! {
                    let data = dir.get_file(format!("{path}/data.ron"))?.contents_utf8()?;
                },
                NodeType::OnlyData => quote! {
                    let data = dir.get_file(format!("{path}.ron"))?.contents_utf8()?;
                },
            }
            .into_token_stream();
            let inner_data_from_dir = match nt {
                NodeType::Name |
                NodeType::Data => quote! {
                    #(s.#option_link_fields = #option_link_types::from_dir(format!("{path}/{}", #option_link_fields_str), dir);)*
                    #(s.#vec_box_link_fields = dir
                        .get_dir(format!("{path}/{}", #vec_box_link_fields_str))
                        .into_iter()
                        .flat_map(|d| d.dirs())
                        .filter_map(|d| #vec_box_link_types::from_dir(d.path().to_string_lossy().to_string(), d))
                        .map(|d| Box::new(d))
                        .collect_vec();)*
                    #(s.#vec_link_fields = dir
                        .get_dir(format!("{path}/{}", #vec_link_fields_str))
                        .into_iter()
                        .flat_map(|d| d.dirs())
                        .filter_map(|d| #vec_link_types::from_dir(d.path().to_string_lossy().to_string(), d))
                        .collect_vec();)*
                },
                NodeType::OnlyData => quote! {},
            }.into_token_stream();
            let data_type_ident = quote! { (#(#all_data_types),*) };
            if let Fields::Named(ref mut fields) = fields {
                fields.named.push(
                    Field::parse_named
                        .parse2(quote! { pub entity: Option<Entity> })
                        .unwrap(),
                );
            }
            quote! {
                #[derive(Component, Clone, Default, Debug)]
                #input
                impl std::fmt::Display for #struct_ident {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(f, "{}", self.kind())
                    }
                }
                impl GetNodeKind for #struct_ident {
                    fn kind(&self) -> NodeKind {
                        NodeKind::#struct_ident
                    }
                }
                impl Show for #struct_ident {
                    fn show(&self, prefix: Option<&str>, ui: &mut Ui) {
                        ui.horizontal(|ui| {
                            for (var, value) in self.get_all_vars() {
                                if var != VarName::name {
                                    value.show(Some(&var.cstr()), ui);
                                }
                            }
                        });
                        #(
                            self.#data_fields.show(None, ui);
                        )*
                    }
                    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool {
                        let mut changed = false;
                        for (var, mut value) in self.get_all_vars() {
                            if value.show_mut(Some(&var.cstr()), ui) {
                                self.set_var(var, value);
                                changed |= true;
                            }
                        }
                        #(
                            changed |= self.#data_fields.show_mut(None, ui);
                        )*
                        changed
                    }
                }
                impl GetVar for #struct_ident {
                    fn get_var(&self, var: VarName) -> Option<VarValue> {
                        match var {
                            #(
                                VarName::#var_fields => return Some(VarValue::#var_types(self.#var_fields.clone())),
                            )*
                            _ => {
                                #(
                                    if let Some(v) = self.#option_link_fields.as_ref().and_then(|l| l.get_var(var)).clone() {
                                        return Some(v);
                                    }
                                )*
                            }
                        };
                        None
                    }
                    fn set_var(&mut self, var: VarName, value: VarValue) {
                        match var {
                            #(
                                VarName::#var_fields => {
                                    self.#var_fields = value.into();
                                }
                            )*
                            _ => {error!("Var {var} not present in {}", self.kind())}
                        }
                    }
                    fn get_all_vars(&self) -> Vec<(VarName, VarValue)> {
                        vec![#(
                            (VarName::#var_fields, VarValue::#var_types(self.#var_fields.clone()))
                        ),*]
                    }
                }
                impl Node for #struct_ident {
                    fn entity(&self) -> Option<Entity> {
                        self.entity
                    }
                    fn get_data(&self) -> String {
                        ron::to_string(&(#(&self.#all_data_fields),*)).unwrap()
                    }
                    fn inject_data(&mut self, data: &str) {
                        match ron::from_str::<#data_type_ident>(data) {
                            Ok(v) => (#(self.#all_data_fields),*) = v,
                            Err(e) => panic!("{} parsing error from {data}: {e}", self.kind()),
                        }
                    }
                    fn from_dir(path: String, dir: &Dir) -> Option<Self> {
                        dbg!(&path);
                        #data_from_dir
                        let mut s = Self::from_data(data);
                        #inner_data_from_dir
                        Some(s)
                    }
                    fn unpack(mut self, entity: Entity, commands: &mut Commands) {
                        debug!("Unpack {self} into {entity}");
                        #on_unpack
                        self.entity = Some(entity);
                        #(
                            if let Some(d) = self.#option_link_fields.take() {
                                d.unpack(entity, commands);
                            }
                        )*
                        #(
                            for d in std::mem::take(&mut self.#vec_link_fields) {
                                let entity = commands.spawn_empty().set_parent(entity).id();
                                d.unpack(entity, commands);
                            }
                        )*
                        #(
                            for d in std::mem::take(&mut self.#vec_box_link_fields) {
                                let entity = commands.spawn_empty().set_parent(entity).id();
                                d.unpack(entity, commands);
                            }
                        )*
                        commands.entity(entity).insert((TransformBundle::default(), VisibilityBundle::default(), self));
                    }
                    fn ui(&self, depth: usize, ui: &mut Ui, world: &World) {
                        let color = NodeState::get_var_world(VarName::color, self.entity.unwrap(), world)
                            .and_then(|c| c.get_color().ok().map(|c| c.c32()));
                        NodeFrame::show(self, depth, color, ui, |ui| {
                            #(
                                if let Some(d) = &self.#option_link_fields {
                                    d.ui(depth + 1, ui, world);
                                } else {
                                    if let Some(c) = world.get::<#option_link_types>(self.entity.unwrap()) {
                                        c.ui(depth + 1, ui, world);
                                    }
                                }
                            )*
                            #(
                                let mut children = self.collect_children::<#vec_link_types>(world);
                                children.extend(self.#vec_link_fields.iter());
                                if !children.is_empty() {
                                    ui.collapsing(#vec_link_fields_str, |ui| {
                                        for (i, c) in children.into_iter().enumerate() {
                                            ui.push_id(i, |ui| {
                                                c.ui(depth + 1, ui, world);
                                            });
                                        }
                                    });
                                }
                            )*
                            #(
                                let mut children = self.collect_children::<#vec_box_link_types>(world);
                                children.extend(self.#vec_box_link_fields.iter().map(Box::as_ref));
                                if !children.is_empty() {
                                    ui.collapsing(#vec_box_link_fields_str, |ui| {
                                        for (i, c) in children.into_iter().enumerate() {
                                            ui.push_id(i, |ui| {
                                                c.ui(depth + 1, ui, world);
                                            });
                                        }
                                    });
                                }
                            )*
                        });
                    }
                }
                impl From<&str> for #struct_ident {
                    fn from(value: &str) -> Self {
                        Self::from_data(value)
                    }
                }
            }
        }
        _ => unimplemented!(),
    };
    result.into()
}

#[proc_macro_attribute]
pub fn node_kinds(_: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as syn::DeriveInput);
    let struct_ident = &input.ident;
    match &mut input.data {
        Data::Enum(DataEnum {
            enum_token: _,
            brace_token: _,
            variants,
        }) => {
            let variants = variants.iter().map(|v| v.ident.clone()).collect_vec();
            quote! {
                #input
                impl NodeKind {
                    pub fn register(self, app: &mut App) {
                        use bevy_trait_query::RegisterExt;
                        match self {
                            #(#struct_ident::#variants => app.register_component_as::<dyn GetVar, #variants>(),)*
                        };
                    }
                    pub fn set_var(self, entity: Entity, var: VarName, value: VarValue, world: &mut World) {
                        match self {
                            #(#struct_ident::#variants => {
                                world.get_mut::<#variants>(entity).unwrap().set_var(var, value);
                            })*
                        }
                    }
                    pub fn show(self, entity: Entity, ui: &mut Ui, world: &World) {
                        match self {
                            #(#struct_ident::#variants => {
                                world.get::<#variants>(entity).unwrap().show(None, ui);
                            })*
                        };
                    }
                    pub fn show_mut(self, entity: Entity, ui: &mut Ui, world: &mut World) {
                        match self {
                            #(#struct_ident::#variants => {
                                world.get_mut::<#variants>(entity).unwrap().show_mut(None, ui);
                            })*
                        };
                    }

                }
            }.into()
        }
        _ => unimplemented!(),
    }
}
