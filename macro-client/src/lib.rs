use itertools::Itertools;
use parse::Parser;
use proc_macro::TokenStream;
use proc_macro2::Span;
use schema::*;
use syn::*;
#[macro_use]
extern crate quote;

#[proc_macro_attribute]
pub fn node(_: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as syn::DeriveInput);
    let struct_ident = &input.ident;

    enum NodeType {
        Name(Ident),
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
                data_types: _,
                data_type_ident: _,
                all_data_fields,
                all_data_types,
            } = parse_node_fields(fields);
            let strings_conversions = strings_conversions(
                &one_fields,
                &one_fields_str,
                &one_types,
                &many_fields,
                &many_fields_str,
                &many_types,
            );
            let nt = if let Some(name_ident) = all_data_fields
                .iter()
                .find(|ident| ident.to_string().contains("name"))
            {
                NodeType::Name(name_ident.clone())
            } else if !one_fields.is_empty() {
                NodeType::Data
            } else {
                NodeType::OnlyData
            };
            let name_quote = match &nt {
                NodeType::Name(ident) => quote! {self.#ident},
                NodeType::Data | NodeType::OnlyData => quote! {""},
            };
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
                        .parse2(quote! { pub owner: u64 })
                        .unwrap(),
                );
                fields.named.insert(
                    0,
                    Field::parse_named.parse2(quote! { pub id: u64 }).unwrap(),
                );
            }
            let common = common_node_fns(
                struct_ident,
                &all_data_fields,
                &all_data_types,
                &one_fields,
                &one_types,
            );
            let common_trait = common_node_trait_fns(struct_ident, &one_types, &many_types);
            let component_fields_load = one_fields
                .iter()
                .map(|i| Ident::new(&format!("{i}_load"), Span::call_site()))
                .collect_vec();
            let component_fields_load_err = one_fields
                .iter()
                .map(|i| Ident::new(&format!("{i}_err"), Span::call_site()))
                .collect_vec();
            let child_fields_load = many_fields
                .iter()
                .map(|i| Ident::new(&format!("{i}_load"), Span::call_site()))
                .collect_vec();
            quote! {
                #[derive(Component, Clone, Debug, Hash)]
                #input
                #common
                impl Default for #struct_ident {
                    fn default() -> Self {
                        Self {
                            id: next_id(),
                            entity: None,
                            owner: 0,
                            #(
                                #one_fields: None,
                            )*
                            #(
                                #many_fields: default(),
                            )*
                            #(
                                #all_data_fields: default(),
                            )*
                        }
                    }
                }
                impl #struct_ident {
                    #(
                        pub fn #component_fields_load<'a>(&'a self, context: &'a Context) -> Option<&'a #one_types> {
                            self.#one_fields.as_ref().or_else(|| {
                                self.entity
                                    .and_then(|e| context.get_node::<#one_types>(e))
                            })
                        }
                        pub fn #component_fields_load_err<'a>(&'a self, context: &'a Context) -> Result<&'a #one_types, ExpressionError> {
                            self.#one_fields.as_ref().or_else(|| {
                                self.entity
                                    .and_then(|e| context.get_node::<#one_types>(e))
                            }).to_e_fn(|| format!("Failed to load {} of {}", #one_fields_str, self.kind()))
                        }
                    )*
                    #(
                        pub fn #child_fields_load<'a>(&'a self, context: &'a Context) -> Vec<&'a #many_types> {
                            if !self.#many_fields.is_empty() {
                                self.#many_fields.iter().collect()
                            } else if let Some(entity) = self.entity {
                                context.children_nodes::<#many_types>(entity).into_iter().sorted_by_key(|n| n.id).collect_vec()
                            } else {
                                default()
                            }
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
                        format!("[tw {self}] [th {}]", #name_quote)
                    }
                }
                impl GetVar for #struct_ident {
                    fn get_own_var(&self, var: VarName) -> Option<VarValue> {
                        match var {
                            #(
                                VarName::#var_fields => Some(self.#var_fields.clone().into()),
                            )*
                            _ => None
                        }
                    }
                    fn get_var(&self, var: VarName, context: &Context) -> Option<VarValue> {
                        if let Some(value) = self.get_own_var(var) {
                            return Some(value);
                        }
                        #(
                            if let Some(v) = self.#one_fields.as_ref()
                                .or_else(|| {
                                    self.entity
                                        .and_then(|e| context.get_node::<#one_types>(e))
                                })
                                .and_then(|d| d.get_var(var, context)).clone() {
                                return Some(v);
                            }
                        )*
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
                                    if let Some(n) = &mut self.#one_fields {
                                        n.set_var(var, value.clone());
                                    }
                                )*
                            }
                        }
                    }
                    fn get_own_vars(&self) -> Vec<(VarName, VarValue)> {
                        vec![
                        #(
                            (VarName::#var_fields, self.#var_fields.clone().into())
                        ),*
                        ]
                    }
                    fn get_vars(&self, context: &Context) -> Vec<(VarName, VarValue)> {
                        let mut vars = self.get_own_vars().into_iter().collect_vec();
                        #(
                            if let Some(d) = self.#one_fields.as_ref().or_else(|| {
                                self.entity
                                    .and_then(|e| context.get_node::<#one_types>(e))
                            }) {
                                vars.extend(d.get_vars(context));
                            }
                        )*
                        vars
                    }
                }
                impl Show for #struct_ident {
                    fn show(&self, context: &Context, ui: &mut Ui) {
                        for (var, value) in self.get_own_vars() {
                            value.show(context, ui);
                        }
                        #(
                            self.#data_fields.show(context, ui);
                        )*
                    }
                    fn show_mut(&mut self, context: &Context, ui: &mut Ui) -> bool {
                        let mut changed = false;
                        #(
                            VarName::#var_fields.cstr().label(ui);
                            changed |= self.#var_fields.show_mut(context, ui);
                        )*
                        #(
                            changed |= self.#data_fields.show_mut(context, ui);
                        )*
                        changed
                    }
                }
                impl Node for #struct_ident {
                    #strings_conversions
                    #common_trait
                    fn id(&self) -> u64 {
                        self.id
                    }
                    fn set_id(&mut self, id: u64) {
                        self.id = id;
                    }
                    fn owner(&self) -> u64 {
                        self.owner
                    }
                    fn set_owner(&mut self, owner: u64) {
                        self.owner = owner;
                    }
                    fn entity(&self) -> Entity {
                        self.entity.expect("Entity not set")
                    }
                    fn get_entity(&self) -> Option<Entity> {
                        self.entity
                    }
                    fn from_dir(path: String, dir: &Dir) -> Option<Self> {
                        let file = dir.get_dir(&path)?.files().next()?;
                        let id = u64::from_str(file.path().file_stem()?.to_str()?).unwrap();
                        let mut d = Self::default();
                        d.inject_data(file.contents_utf8()?).unwrap();
                        d.id = id;
                        #(
                            d.#one_fields = #one_types::from_dir(format!("{path}/{}", #one_fields_str), dir);
                        )*
                        #(
                            d.#many_fields = dir
                                .get_dir(format!("{path}/{}", #many_fields_str))
                                .into_iter()
                                .flat_map(|d| d.dirs())
                                .filter_map(|d| #many_types::from_dir(d.path().to_string_lossy().to_string(), dir))
                                .collect_vec();
                        )*
                        Some(d)
                    }
                    fn to_dir<'a>(&self, path: String) -> &'a [DirEntry<'a>] {
                        let mut entries: Vec<DirEntry> = default();
                        let file = DirEntry::File(File::new(
                            format!("{path}/{}.ron", self.id()).leak(),
                            self.get_data().leak().as_bytes(),
                        ));
                        entries.push(file);
                        #(
                            let child_path = format!("{path}/{}", #one_fields_str);
                            let dir = Dir::new(
                                child_path.clone().leak(),
                                self.#one_fields
                                    .as_ref()
                                    .and_then(|c| Some(c.to_dir(child_path)))
                                    .unwrap_or_default(),
                            );
                            let dir = DirEntry::Dir(dir);
                            entries.push(dir);
                        )*
                        #(
                            let child_path = format!("{path}/{}", #many_fields_str);
                            let dir = Dir::new(
                                child_path.clone().leak(),
                                self.#many_fields
                                    .iter()
                                    .map(|d| {
                                        let path = format!("{child_path}/{}", d.id());
                                        DirEntry::Dir(Dir::new(path.clone().leak(), d.to_dir(path)))
                                    })
                                    .collect_vec()
                                    .leak(),
                            );
                            let dir = DirEntry::Dir(dir);
                            entries.push(dir);
                        )*
                        entries.leak()
                    }
                    fn load_recursive(id: u64) -> Option<Self> {
                        let mut d = Self::load(id)?;
                        #(
                            let kind = #one_types::kind_s().to_string();
                            if let Some(id) = cn()
                                .db
                                .nodes_world()
                                .iter()
                                .find(|n| d.id().is_parent_of(n.id) && n.kind == kind)
                                .map(|n| n.id)
                            {
                                d.#one_fields = #one_types::load_recursive(id);
                            }
                        )*
                        #(
                            let kind = #many_types::kind_s().to_string();
                            d.#many_fields = cn()
                                .db
                                .nodes_world()
                                .iter()
                                .filter_map(|n| {
                                    if d.id().is_parent_of(n.id) && n.kind == kind {
                                        #many_types::load_recursive(n.id)
                                    } else {
                                        None
                                    }
                                })
                                .collect();
                        )*
                        Some(d)
                    }
                    fn pack_entity(entity: Entity, context: &Context) -> Option<Self> {
                        let mut s = context.get_node::<Self>(entity)?.clone();
                        #(
                            s.#one_fields = #one_types::pack_entity(entity, context);
                        )*
                        #(
                            for child in context.get_children(entity) {
                                if let Some(d) = #many_types::pack_entity(child, context) {
                                    s.#many_fields.push(d);
                                }
                            }
                        )*
                        Some(s)
                    }
                    fn unpack_entity(mut self, entity: Entity, world: &mut World) {
                        //debug!("Unpack {}#{:?} into {entity}", self.cstr().to_colored(), self.id);
                        self.entity = Some(entity);
                        if self.id == 0 {
                            self.id = next_id();
                        }
                        world.add_id_link(self.id, entity);
                        #(
                            if let Some(d) = self.#one_fields.take() {
                                let child = world.spawn_empty().id();
                                d.unpack_entity(child, world);
                                child.id(world).add_parent(self.id);
                            }
                        )*
                        #(
                            for d in std::mem::take(&mut self.#many_fields) {
                                let child = world.spawn_empty().id();
                                //debug!("{parent} -> {entity}");
                                d.unpack_entity(child, world);
                                child.id(world).add_parent(self.id);
                            }
                        )*
                        let kind = self.kind();
                        world.entity_mut(entity).insert(self);
                        kind.on_unpack(entity, world);
                    }
                    fn with_components(mut self, context: &Context) -> Self {
                        #(
                            self.#one_fields = #one_types::pack_entity(self.entity(), context);
                        )*
                        self
                    }
                }

                impl DataView for #struct_ident {
                    fn show_value(&self, view_ctx: ViewContext, context: &Context, ui: &mut Ui) {
                        self.show(context, ui);
                    }
                    fn show_title(&self, view_ctx: ViewContext, context: &Context, ui: &mut Ui) -> Response {
                        self.node_title(context, ui)
                    }
                    fn show_collapsed(&self, view_ctx: ViewContext, context: &Context, ui: &mut Ui) -> Response {
                        self.node_collapsed(context, ui)
                    }
                    fn show_value_mut(
                        &mut self,
                        view_ctx: ViewContext,
                        context: &Context,
                        ui: &mut Ui,
                    ) -> bool {
                        self.show_mut(context, ui)
                    }
                    fn context_menu_extra(&self, view_ctx: ViewContext, context: &Context, ui: &mut Ui) {
                        ui.menu_button("publish to incubator", |ui| {

                        });
                    }
                    fn context_menu_extra_mut(&mut self, view_ctx: ViewContext, context: &Context, ui: &mut Ui) -> ViewResponse {
                        let mut view_resp = ViewResponse::default();
                        if let Some(d) = self.replace_context_menu(context, ui) {
                            *self = d;
                            view_resp.changed = true;
                        }
                        view_resp
                    }
                    fn view_children(
                        &self,
                        view_ctx: ViewContext,
                        context: &Context,
                        ui: &mut Ui,
                    ) -> ViewResponse {
                        let mut view_resp = ViewResponse::default();
                        #(
                            if let Some(d) = self.#component_fields_load(context) {
                                view_resp.merge(d.view(view_ctx, context, ui));
                            }
                        )*
                        #(
                            for (i, d) in self.#child_fields_load(context).into_iter().enumerate() {
                                view_resp.merge(d.view(view_ctx.with_id(i), context, ui));
                            }
                        )*
                        view_resp
                    }
                    fn view_children_mut(&mut self, view_ctx: ViewContext, context: &Context, ui: &mut Ui) -> ViewResponse {
                        let mut view_resp = ViewResponse::default();
                        #(
                            if let Some(d) = &mut self.#one_fields {
                                let mut child_resp = d.view_mut(view_ctx.collapsed(true).can_delete(true), context, ui);
                                if child_resp.take_delete_me() {
                                    self.#one_fields = None;
                                }
                                view_resp.merge(child_resp);
                            } else if let Some(mut d) = new_node_btn::<#one_types>(ui, view_ctx) {
                                d.id().add_parent(self.id());
                                view_resp.changed = true;
                                self.#one_fields = Some(d);
                            }
                        )*
                        #(
                            view_resp.merge(self.#many_fields.view_mut(view_ctx.collapsed(true), context, ui));
                        )*
                        view_resp
                    }
                    fn merge_state<'a>(
                        &self,
                        view_ctx: ViewContext,
                        context: &Context<'a>,
                        ui: &mut Ui,
                    ) -> (ViewContext, Context<'a>) {
                        let mut context = context.clone();
                        for (var, value) in self.get_vars(&context) {
                            context.set_var(var, value);
                        }
                        (view_ctx.merge_state(self, ui), context)
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
                                world.get::<#variants>(entity).unwrap().get_own_vars()
                            })*
                        }
                    }
                    pub fn unpack(self, entity: Entity, node: &TNode, world: &mut World) {
                        match self {
                            Self::None => {}
                            #(#struct_ident::#variants => {
                                let mut n = #variants::default();
                                n.inject_data(&node.data);
                                n.id = node.id;
                                n.owner = node.owner;
                                n.unpack_entity(entity, world);
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
                    pub fn remove_component(self, entity: Entity, world: &mut World) {
                        match self {
                            NodeKind::None => {}
                            #(
                                #struct_ident::#variants => {
                                    if let Ok(mut e) = world.get_entity_mut(entity) {
                                        e.remove::<#variants>();
                                    }
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
