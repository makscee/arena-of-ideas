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
                data_types: _,
                data_type_ident: _,
                all_data_fields,
                all_data_types,
            } = parse_node_fields(fields);
            let strings_conversions = strings_conversions(
                &component_link_fields,
                &component_link_fields_str,
                &component_link_types,
                &child_link_fields,
                &child_link_fields_str,
                &child_link_types,
            );
            let no_children = component_link_fields.is_empty() && child_link_fields.is_empty();
            let has_body = if no_children && data_fields.is_empty() {
                quote! {false}
            } else {
                quote! {true}
            };
            let nt = if all_data_fields.contains(&Ident::from_string("name").unwrap()) {
                NodeType::Name
            } else if !component_link_fields.is_empty() {
                NodeType::Data
            } else {
                NodeType::OnlyData
            };
            let name_link = match nt {
                NodeType::Name => quote! {world.add_name_link(self.name.clone(), entity);},
                NodeType::Data | NodeType::OnlyData => quote! {},
            };
            let name_quote = match nt {
                NodeType::Name => quote! {self.name},
                NodeType::Data | NodeType::OnlyData => quote! {""},
            };
            let inner_data_to_dir = match nt {
                NodeType::Name | NodeType::Data => quote! {
                    let mut entries: Vec<DirEntry> = default();
                    #(
                        if let Some(d) = &self.#component_link_fields {
                            let path = format!("{path}/{}", #component_link_fields_str);
                            entries.push(d.to_dir(path));
                        }
                    )*
                    #(
                        {
                            let path = format!("{path}/{}", #child_link_fields_str);
                            entries.push(DirEntry::Dir(Dir::new(
                                path.clone().leak(),
                                self.#child_link_fields
                                    .iter()
                                    .map(|a| a.to_dir(path.clone()))
                                    .collect_vec()
                                    .leak(),
                            )));
                        }
                    )*
                },
                NodeType::OnlyData => quote! {},
            };
            let data_to_dir = match nt {
                NodeType::Name => quote! {
                    let path = format!("{path}/{}", self.name);
                    #inner_data_to_dir
                    DirEntry::Dir(Dir::new(path.leak(), entries.leak()))
                },
                NodeType::Data => quote! {
                    let data = self.get_data();
                    #inner_data_to_dir
                    entries.push(DirEntry::File(File::new(
                        format!("{path}/data.ron").leak(),
                        data.leak().as_bytes(),
                    )));
                    DirEntry::Dir(Dir::new(path.leak(), entries.leak()))
                },
                NodeType::OnlyData => quote! {
                    let data = self.get_data();
                    DirEntry::File(File::new(format!("{path}.ron").leak(), data.leak().as_bytes()))
                },
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
                    #(s.#component_link_fields = #component_link_types::from_dir(format!("{path}/{}", #component_link_fields_str), dir);)*
                    #(s.#child_link_fields = dir
                        .get_dir(format!("{path}/{}", #child_link_fields_str))
                        .into_iter()
                        .flat_map(|d| d.dirs())
                        .filter_map(|d| #child_link_types::from_dir(d.path().to_string_lossy().to_string(), d))
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
                fields.named.push(
                    Field::parse_named
                        .parse2(quote! { pub id: Option<u64> })
                        .unwrap(),
                );
            }
            let insert_unit = if struct_ident.to_string() == "Unit" {
                quote! {vec.push(self);}
            } else {
                default()
            };
            let common = common_node_fns(
                struct_ident,
                &all_data_fields,
                &all_data_types,
                &component_link_fields,
                &component_link_types,
            );
            quote! {
                #[derive(Component, Clone, Default, Debug)]
                #input
                #common
                impl std::fmt::Display for #struct_ident {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(f, "{}", self.kind())
                    }
                }
                impl ToCstr for #struct_ident {
                    fn cstr(&self) -> Cstr {
                        format!("[vd [s {self} [vb {}]]]", #name_quote)
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
                                    if let Some(v) = self.#component_link_fields.as_ref().and_then(|l| l.get_var(var)).clone() {
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
                                    if let Some(n) = &mut self.#component_link_fields {
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
                            if let Some(d) = &self.#component_link_fields {
                                vars.extend(d.get_all_vars());
                            }
                        )*
                        #(
                            for d in &self.#child_link_fields {
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
                            if let Some(d) = &self.#component_link_fields {
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
                            if let Some(d) = &mut self.#component_link_fields {
                                changed |= d.show_mut(None, ui);
                            } else if format!("add [b {}]", #component_link_fields_str).button(ui).clicked() {
                                self.#component_link_fields = Some(default());
                            }
                        )*
                        #(
                            #child_link_fields_str.cstr_c(VISIBLE_DARK).label(ui);
                            let mut delete = None;
                            for (i, d) in self.#child_link_fields.iter_mut().enumerate() {
                                ui.horizontal(|ui| {
                                    if "-".cstr_cs(RED, CstrStyle::Bold).button(ui).clicked() {
                                        delete = Some(i);
                                    }
                                    changed |= d.show_mut(None, ui);
                                });
                            }
                            if let Some(delete) = delete {
                                self.#child_link_fields.remove(delete);
                            }
                            if "+".cstr_cs(VISIBLE_BRIGHT, CstrStyle::Bold).button(ui).clicked() {
                                self.#child_link_fields.push(default());
                            }
                        )*
                        changed
                    }
                }
                impl Node for #struct_ident {
                    #strings_conversions
                    fn id(&self) -> u64 {
                        self.id.unwrap()
                    }
                    fn set_id(&mut self, id: u64) {
                        self.id = Some(id);
                    }
                    fn entity(&self) -> Entity {
                        self.entity.unwrap()
                    }
                    fn get_entity(&self) -> Option<Entity> {
                        self.entity
                    }
                    fn clear_entities(&mut self) {
                        self.entity = None;
                        #(
                            if let Some(d) = &mut self.#component_link_fields {
                                d.clear_entities();
                            }
                        )*
                        #(
                            for d in self.#child_link_fields.iter_mut() {
                                d.clear_entities();
                            }
                        )*
                    }
                    fn from_dir(path: String, dir: &Dir) -> Option<Self> {
                        dbg!(&path);
                        #data_from_dir
                        let mut s = Self::default();
                        s.inject_data(data);
                        #inner_data_from_dir
                        Some(s)
                    }
                    fn to_dir(&self, path: String) -> DirEntry {
                        #data_to_dir
                    }
                    fn load_recursive(id: u64) -> Option<Self> {
                        let mut d = Self::get(id)?;
                        let children = cn()
                            .db
                            .nodes_relations()
                            .iter()
                            .filter(|r| r.parent == id)
                            .map(|r| r.id)
                            .sorted()
                            .collect_vec();
                        #(
                            d.#component_link_fields = #component_link_types::load_recursive(id);
                        )*
                        #(
                            d.#child_link_fields = children
                                .iter()
                                .filter_map(|id| #child_link_types::load_recursive(*id))
                                .collect();
                        )*
                        Some(d)
                    }
                    fn pack(entity: Entity, world: &World) -> Option<Self> {
                        let mut s = world.get::<Self>(entity)?.clone();
                        #(
                            s.#component_link_fields = #component_link_types::pack(entity, world);
                        )*
                        #(
                            for child in get_children(entity, world) {
                                if let Some(d) = #child_link_types::pack(child, world) {
                                    s.#child_link_fields.push(d);
                                }
                            }
                        )*
                        Some(s)
                    }
                    fn unpack(mut self, entity: Entity, world: &mut World) {
                        debug!("Unpack {self} into {entity}");
                        self.entity = Some(entity);
                        if let Some(id) = self.id {
                            world.add_id_link(id, entity);
                        }
                        #name_link
                        #(
                            if let Some(d) = self.#component_link_fields.take() {
                                d.unpack(entity, world);
                            }
                        )*
                        #(
                            for d in std::mem::take(&mut self.#child_link_fields) {
                                let parent = entity;
                                let entity = world.spawn_empty().set_parent(parent).id();
                                debug!("{parent} -> {entity}");
                                d.unpack(entity, world);
                            }
                        )*
                        let kind = self.kind();
                        world.entity_mut(entity).insert(self);
                        kind.on_unpack(entity, world);
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
                    pub fn get_vars(self, entity: Entity, world: &World) -> Vec<(VarName, VarValue)> {
                        match self {
                            Self::None => default(),
                            #(#struct_ident::#variants => {
                                world.get::<#variants>(entity).unwrap().get_vars()
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
                    pub fn unpack(self, entity: Entity, data: &str, id: Option<u64>, world: &mut World) {
                        match self {
                            Self::None => {}
                            #(#struct_ident::#variants => {
                                let mut n = #variants::default();
                                n.inject_data(data);
                                n.id = id;
                                n.unpack(entity, world);
                            })*
                        };
                    }
                }
            }.into()
        }
        _ => unimplemented!(),
    }
}
