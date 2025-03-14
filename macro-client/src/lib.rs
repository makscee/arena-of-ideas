use darling::FromMeta;
use itertools::Itertools;
use parse::Parser;
use proc_macro::TokenStream;
use proc_macro2::Span;
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
                data_types: _,
                data_type_ident: _,
                all_data_fields,
                all_data_types,
            } = parse_node_fields(fields);
            let strings_conversions = strings_conversions(
                &component_fields,
                &component_fields_str,
                &component_types,
                &child_fields,
                &child_fields_str,
                &child_types,
            );
            let no_children = component_fields.is_empty() && child_fields.is_empty();
            let has_body = if no_children && data_fields.is_empty() {
                quote! {false}
            } else {
                quote! {true}
            };
            let nt = if all_data_fields.contains(&Ident::from_string("name").unwrap()) {
                NodeType::Name
            } else if !component_fields.is_empty() {
                NodeType::Data
            } else {
                NodeType::OnlyData
            };
            let default_open = match nt {
                NodeType::Name => quote! {false},
                NodeType::Data | NodeType::OnlyData => quote! {true},
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
                        if let Some(d) = &self.#component_fields {
                            let path = format!("{path}/{}", #component_fields_str);
                            entries.push(d.to_dir(path));
                        }
                    )*
                    #(
                        {
                            let path = format!("{path}/{}", #child_fields_str);
                            entries.push(DirEntry::Dir(Dir::new(
                                path.clone().leak(),
                                self.#child_fields
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
                    #(s.#component_fields = #component_types::from_dir(format!("{path}/{}", #component_fields_str), dir);)*
                    #(s.#child_fields = dir
                        .get_dir(format!("{path}/{}", #child_fields_str))
                        .into_iter()
                        .flat_map(|d| d.dirs())
                        .filter_map(|d| #child_types::from_dir(d.path().to_string_lossy().to_string(), d))
                        .collect_vec();)*
                },
                NodeType::OnlyData => quote! {},
            }.into_token_stream();
            let data_type_ident = quote! { (#(#all_data_types),*) };
            if let Fields::Named(ref mut fields) = fields {
                fields.named.insert(
                    0,
                    Field::parse_named
                        .parse2(quote! { pub entity: Option<Entity> })
                        .unwrap(),
                );
                fields.named.insert(
                    0,
                    Field::parse_named
                        .parse2(quote! { pub parent: Option<u64> })
                        .unwrap(),
                );
                fields.named.insert(
                    0,
                    Field::parse_named
                        .parse2(quote! { pub id: Option<u64> })
                        .unwrap(),
                );
            }
            let common = common_node_fns(
                struct_ident,
                &all_data_fields,
                &all_data_types,
                &component_fields,
                &component_types,
            );
            let common_trait = common_node_trait_fns(struct_ident, &component_types, &child_types);
            let component_link_fields_load = component_fields
                .iter()
                .map(|i| Ident::new(&format!("{i}_load"), Span::call_site()))
                .collect_vec();
            let child_link_fields_load = child_fields
                .iter()
                .map(|i| Ident::new(&format!("{i}_load"), Span::call_site()))
                .collect_vec();
            quote! {
                #[derive(Component, Clone, Default, Debug)]
                #input
                #common
                impl #struct_ident {
                    #(
                        pub fn #component_link_fields_load<'a>(&self, world: &'a World) -> Result<&'a #component_types, ExpressionError> {
                            let entity = self.entity();
                            #component_types::get(entity, world)
                                .to_e_fn(|| format!("{} not found for {}", #component_types::kind_s(), entity))
                        }
                    )*
                    #(
                        pub fn #child_link_fields_load<'a>(&self, world: &'a World) -> Vec<&'a #child_types> {
                            self.collect_children::<#child_types>(world)
                        }
                    )*
                    pub fn find_by_data(
                        #(
                            #all_data_fields: #all_data_types,
                        )*
                    ) -> Option<Self> {
                        let kind = Self::kind_s().to_string();
                        let data = Self {
                            #(
                                #all_data_fields,
                            )*
                            ..default()
                        }.get_data();
                        let n = cn()
                            .db
                            .nodes_world()
                            .iter()
                            .find(|n| n.kind == kind && n.data == data);
                        n.map(|n| n.to_node().unwrap())
                    }
                }
                impl std::fmt::Display for #struct_ident {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(f, "{}", self.kind())
                    }
                }
                impl ToCstr for #struct_ident {
                    fn cstr(&self) -> Cstr {
                        format!("[vd {self} [vb {}]]", #name_quote)
                    }
                }
                impl GetVar for #struct_ident {
                    fn get_var(&self, var: VarName) -> Option<VarValue> {
                        match var {
                            #(
                                VarName::#var_fields => return Some(self.#var_fields.clone().into()),
                            )*
                            _ => {
                                #(
                                    if let Some(v) = self.#component_fields.as_ref().and_then(|l| l.get_var(var)).clone() {
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
                                    if let Some(n) = &mut self.#component_fields {
                                        n.set_var(var, value.clone());
                                    }
                                )*
                            }
                        }
                    }
                    fn get_vars(&self) -> Vec<(VarName, VarValue)> {
                        vec![
                        #(
                            (VarName::#var_fields, self.#var_fields.clone().into())
                        ),*
                        ]
                    }
                    fn get_all_vars(&self) -> Vec<(VarName, VarValue)> {
                        let mut vars = self.get_vars();
                        #(
                            if let Some(d) = &self.#component_fields {
                                vars.extend(d.get_all_vars());
                            }
                        )*
                        #(
                            for d in &self.#child_fields {
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
                    fn inject_data(&mut self, data: &str) -> Result<(), ExpressionError> {
                        match ron::from_str::<#data_type_ident>(data) {
                            Ok(v) => {(#(self.#all_data_fields),*) = v; Ok(())}
                            Err(e) => Err(format!("{} parsing error from {data}: {e}", self.kind()).into()),
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
                    fn default_open(&self) -> bool {
                        #default_open
                    }
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
                            if let Some(d) = &self.#component_fields {
                                d.show(None, context, ui);
                            } else if let Some(d) = self.entity.and_then(|e| context.get_component::<#component_types>(e)) {
                                d.show(None, context, ui);
                            }
                        )*
                        #(
                            if !self.#child_fields.is_empty() {
                                for d in &self.#child_fields {
                                    d.show(None, context, ui);
                                }
                            } else if let Some(e) = self.entity {
                                for (_, d) in context.children_components::<#child_types>(e) {
                                    d.show(None, context, ui);
                                }
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
                            if let Some(d) = &mut self.#component_fields {
                                changed |= d.show_mut(None, ui);
                            } else if format!("add [b {}]", #component_fields_str).button(ui).clicked() {
                                self.#component_fields = Some(default());
                                changed = true;
                            }
                        )*
                        #(
                            #child_fields_str.cstr_c(VISIBLE_DARK).label(ui);
                            let mut delete = None;
                            for (i, d) in self.#child_fields.iter_mut().enumerate() {
                                ui.horizontal(|ui| {
                                    if "-".cstr_cs(RED, CstrStyle::Bold).button(ui).clicked() {
                                        delete = Some(i);
                                    }
                                    changed |= d.show_mut(None, ui);
                                });
                            }
                            if let Some(delete) = delete {
                                self.#child_fields.remove(delete);
                                changed = true;
                            }
                            if "+".cstr_cs(VISIBLE_BRIGHT, CstrStyle::Bold).button(ui).clicked() {
                                self.#child_fields.push(default());
                                changed = true;
                            }
                        )*
                        changed
                    }
                }
                impl Node for #struct_ident {
                    #strings_conversions
                    #common_trait
                    fn id(&self) -> u64 {
                        self.id.unwrap()
                    }
                    fn get_id(&self) -> Option<u64> {
                        self.id
                    }
                    fn set_id(&mut self, id: u64) {
                        self.id = Some(id);
                    }
                    fn parent(&self) -> u64 {
                        self.parent.unwrap()
                    }
                    fn set_parent(&mut self, id: u64) {
                        self.parent = Some(id);
                    }
                    fn entity(&self) -> Entity {
                        self.entity.unwrap()
                    }
                    fn get_entity(&self) -> Option<Entity> {
                        self.entity
                    }
                    fn from_dir_new(parent: u64, path: String, dir: &Dir) -> Option<Self> {
                        let file = dir.get_dir(&path)?.files().next()?;
                        let id = u64::from_str(file.path().file_stem()?.to_str()?).unwrap();
                        let mut d = Self::default();
                        d.inject_data(file.contents_utf8()?);
                        d.id = Some(id);
                        d.parent = Some(parent);
                        #(
                            d.#component_fields = #component_types::from_dir_new(id, format!("{path}/{}", #component_fields_str), dir);
                        )*
                        #(
                            d.#child_fields = dir
                                .get_dir(format!("{path}/{}", #child_fields_str))
                                .into_iter()
                                .flat_map(|d| d.dirs())
                                .filter_map(|d| #child_types::from_dir_new(id, d.path().to_string_lossy().to_string(), dir))
                                .collect_vec();
                        )*
                        Some(d)
                    }
                    fn from_dir(path: String, dir: &Dir) -> Option<Self> {
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
                        let mut d = Self::load(id)?;
                        #(
                            let kind = #component_types::kind_s().to_string();
                            if let Some(id) = cn()
                                .db
                                .nodes_world()
                                .iter()
                                .find(|n| n.parent == d.id() && n.kind == kind)
                                .map(|n| n.id)
                            {
                                d.#component_fields = #component_types::load_recursive(id);
                            }
                        )*
                        #(
                            let kind = #child_types::kind_s().to_string();
                            d.#child_fields = cn()
                                .db
                                .nodes_world()
                                .iter()
                                .filter_map(|n| {
                                    if n.parent == d.id() && n.kind == kind {
                                        #child_types::load_recursive(n.id)
                                    } else {
                                        None
                                    }
                                })
                                .collect();
                        )*
                        Some(d)
                    }
                    fn pack(entity: Entity, world: &World) -> Option<Self> {
                        let mut s = world.get::<Self>(entity)?.clone();
                        #(
                            s.#component_fields = #component_types::pack(entity, world);
                        )*
                        #(
                            for child in get_children(entity, world) {
                                if let Some(d) = #child_types::pack(child, world) {
                                    s.#child_fields.push(d);
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
                            if let Some(d) = self.#component_fields.take() {
                                d.unpack(entity, world);
                            }
                        )*
                        #(
                            for d in std::mem::take(&mut self.#child_fields) {
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
                    fn fill_from_incubator(mut self) -> Self {
                        #(
                            self.#component_fields = self.find_incubator_component();
                            self.#component_fields = self.#component_fields.map(|n| n.fill_from_incubator());
                        )*
                        #(
                            self.#child_fields = self.collect_incubator_children();
                            self.#child_fields = self
                                .#child_fields
                                .into_iter()
                                .map(|n| n.fill_from_incubator())
                                .collect();
                        )*
                        self
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
                    pub fn show_data(self, data: &str, context: &Context, ui: &mut Ui) {
                        match self {
                            Self::None => {}
                            #(#struct_ident::#variants => {
                                let mut d = #variants::default();
                                d.inject_data(data).log();
                                d.show(None, context, ui);
                            })*
                        }
                    }
                    pub fn show_data_mut(self, data: &mut String, ui: &mut Ui) {
                        match self {
                            Self::None => {}
                            #(#struct_ident::#variants => {
                                let mut d = #variants::default();
                                d.inject_data(data).log();
                                d.show_mut(None, ui);
                                *data = d.get_data();
                            })*
                        }
                    }
                    pub fn show_tnodes(self, nodes: &Vec<TNode>, context: &Context, ui: &mut Ui) {
                        match self {
                            Self::None => {}
                            #(#struct_ident::#variants => {
                                let mut d = #variants::from_tnodes(nodes[0].id, &nodes).unwrap();
                                d.show(None, context, ui);
                            })*
                        }
                    }
                    pub fn show_tnodes_mut(self, nodes: &mut Vec<TNode>, ui: &mut Ui) {
                        match self {
                            Self::None => {}
                            #(#struct_ident::#variants => {
                                let mut d = #variants::from_tnodes(nodes[0].id, &nodes).unwrap();
                                if d.show_mut(None, ui) {
                                    d.reassign_ids(&mut 0);
                                    *nodes = d.to_tnodes();
                                }
                            })*
                        }
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
                    pub fn is_component(self, child: NodeKind) -> bool {
                        match self {
                            NodeKind::None => false,
                            #(
                                #struct_ident::#variants => {
                                    #variants::component_kinds().contains(&child)
                                }
                            )*
                        }
                    }
                    pub fn default_data(self) -> String {
                        match self {
                            NodeKind::None => unimplemented!(),
                            #(
                                #struct_ident::#variants => {
                                    #variants::default().get_data()
                                }
                            )*
                        }
                    }
                    pub fn default_tnode(self) -> TNode {
                        match self {
                            NodeKind::None => unimplemented!(),
                            #(
                                #struct_ident::#variants => {
                                    let mut d = #variants::default();
                                    d.set_id(0);
                                    d.set_parent(0);
                                    d.to_tnode()
                                }
                            )*
                        }
                    }
                }
            }.into()
        }
        _ => unimplemented!(),
    }
}
