use darling::FromMeta;
use itertools::Itertools;
use parse::Parser;
use proc_macro::TokenStream;
use quote::ToTokens;
use schema::*;
use syn::*;
#[macro_use]
extern crate quote;

#[proc_macro_attribute]
pub fn node(_: TokenStream, item: TokenStream) -> TokenStream {
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
            let ParsedNodeFields {
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
                data_fields_str,
                data_types: _,
                data_type_ident: _,
                all_data_fields,
                all_data_types,
            } = parse_node_fields(fields);
            let strings_conversions = strings_conversions(
                &option_link_fields,
                &option_link_fields_str,
                &option_link_types,
                &vec_link_fields,
                &vec_link_fields_str,
                &vec_link_types,
                &vec_box_link_fields,
                &vec_box_link_fields_str,
                &vec_box_link_types,
            );
            let no_children = option_link_fields.is_empty()
                && vec_link_fields.is_empty()
                && vec_box_link_fields.is_empty();
            let has_body = if no_children && data_fields.is_empty() {
                quote! {false}
            } else {
                quote! {true}
            };
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
            let insert_unit = if struct_ident.to_string() == "Unit" {
                quote! {vec.push(self);}
            } else {
                default()
            };
            quote! {
                #[derive(Component, Clone, Default, Debug)]
                #input
                impl std::fmt::Display for #struct_ident {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(f, "{}", self.kind())
                    }
                }
                impl ToCstr for #struct_ident {
                    fn cstr(&self) -> Cstr {
                        format!("[vd [s {self}]]")
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
                            _ => {
                                #(
                                    if let Some(n) = &mut self.#option_link_fields {
                                        n.set_var(var, value.clone());
                                    }
                                )*
                            }
                        }
                    }
                    fn get_vars(&self) -> Vec<(VarName, VarValue)> {
                        vec![
                        #(
                            (VarName::#var_fields, VarValue::#var_types(self.#var_fields.clone()))
                        ),*
                        ]
                    }
                    fn get_all_vars(&self) -> Vec<(VarName, VarValue)> {
                        let mut vars = self.get_vars();
                        #(
                            if let Some(d) = &self.#option_link_fields {
                                vars.extend(d.get_all_vars());
                            }
                        )*
                        #(
                            for d in &self.#vec_link_fields {
                                vars.extend(d.get_all_vars());
                            }
                        )*
                        #(
                            for d in &self.#vec_box_link_fields {
                                vars.extend(d.get_all_vars());
                            }
                        )*
                        vars
                    }
                }
                impl StringData for #struct_ident {
                    fn get_data(&self) -> String {
                        ron::to_string(&(#(&self.#all_data_fields),*)).unwrap()
                    }
                    fn inject_data(&mut self, data: &str) {
                        match ron::from_str::<#data_type_ident>(data) {
                            Ok(v) => (#(self.#all_data_fields),*) = v,
                            Err(e) => panic!("{} parsing error from {data}: {e}", self.kind()),
                        }
                    }
                }
                impl Inject for #struct_ident {
                    fn move_inner(&mut self, source: &mut Self) {}
                    fn wrapper() -> Self {
                        Self::default()
                    }
                }
                impl Injector<Self> for #struct_ident {
                    fn get_inner_mut(&mut self) -> Vec<&mut Box<Self>> {
                        default()
                    }
                    fn get_inner(&self) -> Vec<&Box<Self>> {
                        default()
                    }
                }
                impl DataFramed for #struct_ident {
                    fn has_header(&self) -> bool {
                        true
                    }
                    fn has_body(&self) -> bool {
                        #has_body
                    }
                    fn show_header(&self, context: &Context, ui: &mut Ui) {
                        if !#has_body {
                            ui.horizontal(|ui| {
                                for (var, value) in self.get_vars() {
                                    value.show(Some(&var.cstr()), context, ui);
                                }
                            });
                        }
                    }
                    fn show_header_mut(&mut self, ui: &mut Ui) -> bool {
                        if #has_body {
                            return false;
                        }
                        let mut changed = false;
                        #(
                            VarName::#var_fields.cstr().label(ui);
                            changed |= self.#var_fields.show_mut(None, ui);
                        )*
                        changed
                    }
                    fn show_body(&self, context: &Context, ui: &mut Ui) {
                        for (var, value) in self.get_vars() {
                            ui.horizontal(|ui| {
                                value.show(Some(&var.cstr()), context, ui);
                            });
                        }
                        #(
                            self.#data_fields.show(Some(#data_fields_str), context, ui);
                        )*
                        #(
                            if let Some(d) = &self.#option_link_fields {
                                d.show(None, context, ui);
                            }
                        )*
                    }
                    fn show_body_mut(&mut self, ui: &mut Ui) -> bool {
                        let mut changed = false;
                        #(
                            VarName::#var_fields.cstr().label(ui);
                            changed |= self.#var_fields.show_mut(None, ui);
                        )*
                        #(
                            changed |= self.#data_fields.show_mut(Some(#data_fields_str), ui);
                        )*
                        #(
                            if let Some(d) = &mut self.#option_link_fields {
                                changed |= d.show_mut(None, ui);
                            }
                        )*
                        changed
                    }
                }
                impl Node for #struct_ident {
                    #strings_conversions
                    fn entity(&self) -> Option<Entity> {
                        self.entity
                    }
                    fn from_dir(path: String, dir: &Dir) -> Option<Self> {
                        dbg!(&path);
                        #data_from_dir
                        let mut s = Self::default();
                        s.inject_data(data);
                        #inner_data_from_dir
                        Some(s)
                    }
                    fn from_table(domain: NodeDomain, id: u64) -> Option<Self> {
                        let data = domain.find_by_key(&Self::kind_s().key(id))?.data;
                        let mut d = Self::default();
                        d.inject_data(&data);
                        let children = cn()
                            .db
                            .nodes_relations()
                            .iter()
                            .filter(|r| r.parent == id)
                            .map(|r| r.id)
                            .sorted()
                            .collect_vec();
                        #(
                            d.#option_link_fields = #option_link_types::from_table(domain, id);
                        )*
                        #(
                            d.#vec_link_fields = children
                                .iter()
                                .filter_map(|id| #vec_link_types::from_table(domain, *id))
                                .collect();
                        )*
                        #(
                            d.#vec_box_link_fields = children
                                .iter()
                                .filter_map(|id| #vec_box_link_types::from_table(domain, *id))
                                .map(|d| Box::new(d))
                                .collect();
                        )*
                        Some(d)
                    }
                    fn unpack(mut self, entity: Entity, commands: &mut Commands) {
                        debug!("Unpack {self} into {entity}");
                        self.kind().on_unpack(entity, commands);
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
                        commands.entity(entity).insert(self);
                    }
                    fn collect_units_vec<'a>(&'a self, vec: &mut Vec<&'a Unit>) {
                        #insert_unit
                        #(
                            if let Some(d) = &self.#option_link_fields {
                                d.collect_units_vec(vec);
                            }
                        )*
                        #(
                            for d in &self.#vec_link_fields {
                                d.collect_units_vec(vec);
                            }
                        )*
                        #(
                            for d in &self.#vec_box_link_fields {
                                d.collect_units_vec(vec);
                            }
                        )*
                    }
                    fn ui(&self, depth: usize, context: &Context, ui: &mut Ui) {
                        let color = context.get_var(VarName::color)
                            .and_then(|c| c.get_color()).ok();
                        NodeFrame::show(self, depth, color, ui, |ui| {
                            self.show(None, context, ui);
                            #(
                                if let Some(d) = &self.#option_link_fields {
                                    d.ui(depth + 1, context, ui);
                                } else {
                                    if let Some(c) = context.get_component::<#option_link_types>(self.entity.unwrap()) {
                                        c.ui(depth + 1, context, ui);
                                    }
                                }
                            )*
                            #(
                                let mut children = self.collect_children::<#vec_link_types>(context).into_iter().map(|(_,c)| c).collect_vec();
                                children.extend(self.#vec_link_fields.iter());
                                if !children.is_empty() {
                                    ui.collapsing(#vec_link_fields_str, |ui| {
                                        for (i, c) in children.into_iter().enumerate() {
                                            ui.push_id(i, |ui| {
                                                c.ui(depth + 1, context, ui);
                                            });
                                        }
                                    });
                                }
                            )*
                            #(
                                let mut children = self.collect_children::<#vec_box_link_types>(context).into_iter().map(|(_,c)| c).collect_vec();
                                children.extend(self.#vec_box_link_fields.iter().map(Box::as_ref));
                                if !children.is_empty() {
                                    ui.collapsing(#vec_box_link_fields_str, |ui| {
                                        for (i, c) in children.into_iter().enumerate() {
                                            ui.push_id(i, |ui| {
                                                c.ui(depth + 1, context, ui);
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
                        let mut d = Self::default();
                        d.inject_data(value);
                        d
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
            let variants = variants
                .iter()
                .map(|v| v.ident.clone())
                .filter(|v| v != "None")
                .collect_vec();
            quote! {
                #input
                impl NodeKind {
                    pub fn register(self, app: &mut App) {
                        use bevy_trait_query::RegisterExt;
                        match self {
                            Self::None => {}
                            #(#struct_ident::#variants => {app.register_component_as::<dyn GetVar, #variants>();})*
                        };
                    }
                    pub fn register_world(self, world: &mut World) {
                        use bevy_trait_query::RegisterExt;
                        match self {
                            Self::None => {}
                            #(#struct_ident::#variants => {world.register_component_as::<dyn GetVar, #variants>();})*
                        };
                    }
                    pub fn set_var(self, entity: Entity, var: VarName, value: VarValue, world: &mut World) {
                        match self {
                            Self::None => {}
                            #(#struct_ident::#variants => {
                                world.get_mut::<#variants>(entity).unwrap().set_var(var, value);
                            })*
                        }
                    }
                    pub fn show(self, entity: Entity, ui: &mut Ui, world: &World) {
                        let context = Context::new_world(world).set_owner(entity).take();
                        match self {
                            Self::None => {}
                            #(#struct_ident::#variants => {
                                context.get_component::<#variants>(entity).unwrap().show(None, &context, ui);
                            })*
                        };
                    }
                    pub fn show_mut(self, entity: Entity, ui: &mut Ui, world: &mut World) {
                        match self {
                            Self::None => {}
                            #(#struct_ident::#variants => {
                                world.get_mut::<#variants>(entity).unwrap().show_mut(None, ui);
                            })*
                        };
                    }
                    pub fn unpack(self, entity: Entity, data: &str, commands: &mut Commands) {
                        match self {
                            Self::None => {}
                            #(#struct_ident::#variants => {
                                let mut n = #variants::default();
                                n.inject_data(data);
                                n.unpack(entity, commands);
                            })*
                        };
                    }
                }
            }.into()
        }
        _ => unimplemented!(),
    }
}
