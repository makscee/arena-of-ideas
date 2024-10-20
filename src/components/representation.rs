use egui::CollapsingHeader;

use super::*;

#[derive(
    Asset, Serialize, TypePath, Deserialize, Debug, Component, Resource, Clone, Default, PartialEq,
)]
#[serde(deny_unknown_fields)]
pub struct Representation {
    pub material: RepresentationMaterial,
    #[serde(default)]
    pub children: Vec<Box<Representation>>,
    #[serde(default)]
    pub mapping: IndexMap<VarName, Expression>,
    #[serde(default)]
    pub count: usize,
    #[serde(skip)]
    pub entity: Option<Entity>,
    #[serde(skip)]
    pub material_entities: Vec<Entity>,
}

impl Representation {
    pub fn unpack(mut self, entity: Entity, world: &mut World) -> Entity {
        let rep = self.unpack_materials(entity, world);
        world.entity_mut(entity).insert(self);
        rep
    }
    fn unpack_materials(&mut self, entity: Entity, world: &mut World) -> Entity {
        world
            .entity_mut(entity)
            .insert(TransformBundle::default())
            .insert(VisibilityBundle::default());
        if !self.mapping.is_empty() {
            VarState::default().attach(entity, 0, world);
        }
        self.entity = Some(entity);
        for i in 0..self.count.max(1) {
            let entity = world.spawn_empty().set_parent(entity).id();
            self.material.unpack(entity, world);
            VarState::new_with(VarName::Index, VarValue::Int(i as i32)).attach(entity, 0, world);
            world.get_mut::<Transform>(entity).unwrap().translation.z += 0.001 * i as f32;
            self.material_entities.push(entity);
        }
        self.unpack_children(world);
        let entity = *self.material_entities.first().unwrap();
        // if is_dev_mode() {
        //     debug!("unpack material {} {entity}", self.material);
        // }
        entity
    }
    fn unpack_children(&mut self, world: &mut World) {
        let parent = *self.material_entities.first().unwrap();
        for (i, child) in self.children.iter_mut().enumerate() {
            let entity = world.spawn_empty().set_parent(parent).id();
            child.unpack_materials(entity, world);
            world.get_mut::<Transform>(entity).unwrap().translation.z += (i + 1) as f32;
        }
    }

    pub fn pack(entity: Entity, world: &World) -> Self {
        let mut rep = world.get::<Representation>(entity).unwrap().clone();
        rep.material_entities.clear();
        rep.entity = None;
        rep
    }
    pub fn update(self, entity: Entity, context: Context, world: &mut World) {
        let t = gt().play_head();
        {
            let visible = context.get_bool(VarName::Visible, world).unwrap_or(true);
            let visible = visible && context.get_birth(world).unwrap_or_default() < t;
            RepresentationMaterial::set_visible(entity, visible, world);
            if !visible {
                return;
            }
        }
        context.apply_transform(&[VarName::Position, VarName::Scale], world);
        for (i, entity) in self.material_entities.iter().enumerate() {
            let context = context
                .clone()
                .set_owner(*entity)
                .set_var(VarName::Index, (i as i32).into())
                .take();
            self.apply_mapping(*entity, world);
            context.apply_transform(&[VarName::Rotation, VarName::Scale, VarName::Offset], world);
            self.material.update(*entity, &context, world);
        }
        for child in self.children {
            let entity = child.entity.unwrap();
            child.update(entity, Context::new_play(entity), world);
        }
    }

    fn apply_mapping(&self, entity: Entity, world: &mut World) {
        let context = Context::new_play(entity);
        let mapping: HashMap<VarName, VarValue> =
            HashMap::from_iter(self.mapping.iter().filter_map(|(var, value)| {
                match value.get_value(&context, world) {
                    Ok(value) => Some((*var, value)),
                    Err(_) => {
                        // debug!("{e}");
                        None
                    }
                }
            }));
        if mapping.is_empty() {
            return;
        }
        let mut state = VarState::get_mut(entity, world);
        for (var, value) in mapping {
            state.init(var, value);
        }
    }

    pub fn despawn_all(world: &mut World) {
        for entity in world
            .query_filtered::<Entity, With<Representation>>()
            .iter(world)
            .collect_vec()
        {
            if let Some(entity) = world.get_entity_mut(entity) {
                entity.despawn_recursive()
            }
        }
    }
}

impl ShowEditor for Representation {
    fn get_variants() -> impl Iterator<Item = Self> {
        None.into_iter()
    }
    fn show_children(&mut self, context: &Context, world: &mut World, ui: &mut Ui) {
        if Button::click("+").ui(ui).clicked() {
            self.children.push(default());
        }
        let mut to_remove = None;
        for (i, child) in self.children.iter_mut().enumerate() {
            ui.push_id(i, |ui| {
                if Button::click("-").red(ui).ui(ui).clicked() {
                    to_remove = Some(i);
                }
                child.show_node("", context, world, ui);
            });
        }
        if let Some(i) = to_remove {
            self.children.remove(i);
        }
    }
    fn show_content(&mut self, context: &Context, world: &mut World, ui: &mut Ui) {
        DragValue::new(&mut self.count)
            .range(0..=20)
            .prefix("count:")
            .ui(ui);
        CollapsingHeader::new(format!("Mapping ({})", self.mapping.len()))
            .id_source("mapping")
            .show(ui, |ui| {
                if Button::click("+").ui(ui).clicked() {
                    self.mapping.insert_before(0, default(), default());
                }
                let mut move_var: Option<(VarName, VarName)> = None;
                let mut remove_var = None;
                for (var, value) in &mut self.mapping {
                    ui.push_id(var, |ui| {
                        ui.horizontal(|ui| {
                            let mut new_var = *var;
                            ui.vertical(|ui| {
                                if var_selector(&mut new_var, ui) {
                                    move_var = Some((*var, new_var));
                                }
                                if Button::click("-").red(ui).ui(ui).clicked() {
                                    remove_var = Some(*var);
                                }
                            });
                            value.show_node("", context, world, ui)
                        });
                    });
                }
                if let Some((from, to)) = move_var {
                    let index = self.mapping.get_index_of(&from).unwrap();
                    let value = self.mapping.shift_remove(&from).unwrap();
                    let (index, old) = self.mapping.insert_before(index, to, value);
                    if let Some(old) = old {
                        self.mapping.insert_before(index, from, old);
                    }
                }
                if let Some(var) = remove_var {
                    self.mapping.shift_remove(&var);
                }
            });
        self.material.show_node("", context, world, ui);
    }
    fn get_inner_mut(&mut self) -> Vec<&mut Box<Self>> {
        default()
    }
}

impl ToCstr for Representation {
    fn cstr(&self) -> Cstr {
        self.material.as_ref().cstr()
    }
}
