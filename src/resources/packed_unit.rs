use crate::module_bindings::{StatusCharges, TableUnit};

use super::*;

#[derive(Asset, Deserialize, Serialize, TypePath, Debug, Clone, PartialEq, Default)]
#[serde(deny_unknown_fields)]
pub struct PackedUnit {
    #[serde(default = "default_text")]
    pub name: String,
    #[serde(default = "default_one")]
    pub hp: i32,
    #[serde(default)]
    pub atk: i32,
    #[serde(default = "default_one")]
    pub stacks: i32,
    #[serde(default = "default_one")]
    pub level: i32,
    #[serde(default = "default_houses")]
    pub houses: String,
    #[serde(default)]
    pub trigger: Trigger,
    #[serde(default)]
    pub representation: Representation,
    #[serde(default)]
    pub state: VarState,
    #[serde(default)]
    pub statuses: Vec<(String, i32)>,
}

fn default_houses() -> String {
    "Default".to_owned()
}
fn default_text() -> String {
    "empty".to_owned()
}
fn default_one() -> i32 {
    1
}

pub const LOCAL_TRIGGER: &str = "_local";

impl PackedUnit {
    pub fn unpack(mut self, parent: Entity, slot: Option<usize>, world: &mut World) -> Entity {
        let entity = world.spawn_empty().set_parent(parent).id();
        let draggable = {
            let faction = VarState::get(parent, world)
                .get_faction(VarName::Faction)
                .unwrap();
            faction.eq(&Faction::Team) || faction.eq(&Faction::Shop)
        };

        self.state = self.generate_state(world);
        self.state.init(
            VarName::Slot,
            VarValue::Int(slot.unwrap_or_default() as i32),
        );
        self.state.clone().attach(entity, world);
        if !SkipVisual::active(world) {
            world.entity_mut(entity).insert(TextColumn::new(entity));
            self.representation.unpack(entity, world);
            let entity = Options::get_unit_rep(world)
                .clone()
                .unpack(world.spawn_empty().set_parent(entity).id(), world);
            let mut emut = world.entity_mut(entity);
            emut.get_mut::<Transform>().unwrap().translation.z += 100.0;
            emut.insert(PickableBundle::default())
                .insert(On::<Pointer<Move>>::run(UnitPlugin::hover_unit))
                .insert(On::<Pointer<Out>>::run(UnitPlugin::unhover_unit));
            if draggable {
                emut.insert(On::<Pointer<DragStart>>::run(UnitPlugin::drag_unit_start))
                    .insert(On::<Pointer<DragEnd>>::run(UnitPlugin::drag_unit_end))
                    .insert(On::<Pointer<Drag>>::run(UnitPlugin::drag_unit));
            }
        }
        Status {
            name: LOCAL_TRIGGER.to_owned(),
            trigger: self.trigger.clone(),
        }
        .spawn(world)
        .insert(
            VarState::new_with(VarName::Charges, VarValue::Int(1))
                .init(VarName::Name, VarValue::String("_local".to_owned()))
                .take(),
        )
        .set_parent(entity);
        for (status, charges) in self.statuses.iter() {
            match Status::change_charges(status, entity, *charges, world) {
                Ok(entity) => Status::refresh_status_mapping(entity, world),
                Err(_) => {}
            }
        }
        if VarState::get_mut(parent, world)
            .get_faction(VarName::Faction)
            .unwrap()
            == Faction::Team
        {
            world.entity_mut(entity).insert(ActiveTeam);
        }
        world
            .entity_mut(entity)
            .insert((Name::new(self.name.clone()), Unit));
        debug!("Unpacked unit {entity:?} {}", self.name);
        entity
    }

    pub fn generate_state(&self, world: &World) -> VarState {
        let mut state = self.state.clone();
        state
            .init(VarName::Hp, VarValue::Int(self.hp))
            .init(VarName::Atk, VarValue::Int(self.atk))
            .init(VarName::Stacks, VarValue::Int(self.stacks))
            .init(VarName::Level, VarValue::Int(self.level))
            .init(VarName::Houses, VarValue::String(self.houses.clone()))
            .init(VarName::Name, VarValue::String(self.name.clone()))
            .init(VarName::Position, VarValue::Vec2(default()))
            .init(VarName::Index, VarValue::Int(0))
            .init(VarName::Dmg, VarValue::Int(0));
        self.trigger.inject_description(&mut state);
        let house_colors = self
            .houses
            .split('+')
            .map(|h| Pools::get_house_color(h, world).unwrap_or(Color::FUCHSIA))
            .collect_vec();
        state
            .init(VarName::HouseColor1, VarValue::Color(house_colors[0]))
            .init(VarName::Color, VarValue::Color(house_colors[0]));
        if let Some(color) = house_colors.get(1) {
            state.init(VarName::HouseColor2, VarValue::Color(*color));
        }
        if let Some(color) = house_colors.get(2) {
            state.init(VarName::HouseColor3, VarValue::Color(*color));
        }
        state
    }

    pub fn get_representation_entity(entity: Entity, world: &World) -> Option<Entity> {
        world
            .get::<Children>(entity)
            .unwrap()
            .into_iter()
            .find(|x| world.get::<Representation>(**x).is_some())
            .copied()
    }

    pub fn pack(entity: Entity, world: &World) -> Self {
        let representation = Representation::pack(entity, world);
        let mut state = VarState::get(entity, world).clone();
        let hp = state.get_int(VarName::Hp).unwrap();
        let atk = state.get_int(VarName::Atk).unwrap();
        let stacks = state.get_int(VarName::Stacks).unwrap();
        let level = state.get_int(VarName::Level).unwrap();
        let name = state.get_string(VarName::Name).unwrap();
        let houses = state.get_string(VarName::Houses).unwrap();
        let mut trigger = None;
        let mut statuses = Vec::default();
        for entity in Status::collect_unit_statuses(entity, world) {
            let status = world.get::<Status>(entity).unwrap();
            if status.name.eq(LOCAL_TRIGGER) {
                trigger = Some(status.trigger.clone());
            } else {
                statuses.push((
                    status.name.to_owned(),
                    VarState::get(entity, world)
                        .get_int(VarName::Charges)
                        .unwrap(),
                ));
            }
        }
        let trigger = trigger.unwrap();
        state
            .clear_value(VarName::Hp)
            .clear_value(VarName::Atk)
            .clear_value(VarName::Level)
            .clear_value(VarName::Stacks)
            .clear_value(VarName::Name)
            .clear_value(VarName::EffectDescription)
            .clear_value(VarName::TargetDescription)
            .clear_value(VarName::TriggerDescription)
            .clear_value(VarName::Houses)
            .simplify();

        Self {
            hp,
            atk,
            houses,
            name,
            trigger,
            representation,
            state,
            statuses,
            stacks,
            level,
        }
    }

    fn fuse_base(target: &Self, source: &Self, trigger: Trigger, world: &World) -> Self {
        let mut fused = target.clone();
        fused.representation.children.push(Box::new({
            let mut rep = source.representation.clone();
            rep.mapping.insert(
                VarName::Color,
                Expression::Value(VarValue::Color(
                    Pools::get_house_color(source.houses.split('+').next().unwrap(), world)
                        .unwrap(),
                )),
            );
            rep
        }));
        fused.hp = fused.hp.max(source.hp);
        fused.atk = fused.atk.max(source.atk);
        fused.houses = format!("{}+{}", target.houses, source.houses);
        fused.name = format!("{}+{}", fused.name, source.name);
        fused.level = 1;
        fused.stacks = 1;
        fused.trigger = trigger;
        fused
    }

    pub fn fuse(target: Self, source: Self, world: &World) -> Vec<Self> {
        let mut result: Vec<Self> = default();
        let t_target = &target.trigger;
        let t_source = &source.trigger;

        match (t_target, t_source) {
            (
                Trigger::Fire {
                    triggers: trigger_a,
                    targets: target_a,
                    effects: effect_a,
                },
                Trigger::Fire {
                    triggers: trigger_b,
                    targets: target_b,
                    effects: effect_b,
                },
            ) => {
                {
                    let trigger = Trigger::Fire {
                        triggers: Vec::from_iter(
                            trigger_a.iter().cloned().chain(trigger_b.iter().cloned()),
                        ),
                        targets: target_a.clone(),
                        effects: effect_a.clone(),
                    };
                    result.push(Self::fuse_base(&target, &source, trigger, world))
                }
                {
                    let trigger = Trigger::Fire {
                        triggers: trigger_a.clone(),
                        targets: Vec::from_iter(
                            target_a.iter().cloned().chain(target_b.iter().cloned()),
                        ),
                        effects: effect_a.clone(),
                    };
                    result.push(Self::fuse_base(&target, &source, trigger, world))
                }
                {
                    let trigger = Trigger::Fire {
                        triggers: trigger_a.clone(),
                        targets: target_a.clone(),
                        effects: Vec::from_iter(
                            effect_a.iter().cloned().chain(effect_b.iter().cloned()),
                        ),
                    };
                    result.push(Self::fuse_base(&target, &source, trigger, world))
                }
            }
            _ => {
                let trigger =
                    Trigger::List([Box::new(t_target.clone()), Box::new(t_source.clone())].into());
                result.push(Self::fuse_base(&target, &source, trigger, world));
            }
        }

        result
    }

    pub fn statuses_string(&self) -> String {
        self.statuses
            .iter()
            .map(|(name, charges)| format!("{name} ({charges})"))
            .join(",")
    }

    pub fn show_editor(&mut self, entity: Entity, ui: &mut Ui, world: &mut World) {
        let style = ui.style_mut();
        style.override_text_style = Some(TextStyle::Small);
        style.drag_value_text_style = TextStyle::Small;
        style.visuals.widgets.inactive.bg_stroke = Stroke {
            width: 1.0,
            color: dark_gray(),
        };
        ui.horizontal(|ui| {
            let name = &mut self.name;
            ui.label("name:");
            TextEdit::singleline(name).desired_width(60.0).ui(ui);
            let atk = &mut self.atk;
            ui.label("atk:");
            DragValue::new(atk).clamp_range(0..=99).ui(ui);
            let hp = &mut self.hp;
            ui.label("hp:");
            DragValue::new(hp).clamp_range(0..=99).ui(ui);
            let lvl = &mut self.level;
            ui.label("lvl:");
            DragValue::new(lvl).clamp_range(1..=99).ui(ui);
        });
        ui.horizontal(|ui| {
            let houses: HashMap<String, Color> = HashMap::from_iter(
                Pools::get(world)
                    .houses
                    .iter()
                    .map(|(k, v)| (k.clone(), v.color.clone().into())),
            );
            ui.label("house:");
            let house = &mut self.houses;
            ComboBox::from_id_source("house")
                .selected_text(house.clone())
                .width(140.0)
                .show_ui(ui, |ui| {
                    for (h, _) in houses.into_iter().sorted_by_key(|(k, _)| k.clone()) {
                        ui.selectable_value(house, h.clone(), h.clone());
                    }
                });
        });

        let context = &Context::from_owner(entity, world);
        ui.horizontal(|ui| {
            let trigger = &mut self.trigger;
            match trigger {
                Trigger::Fire {
                    triggers,
                    targets,
                    effects,
                } => {
                    CollapsingHeader::new("Triggers")
                        .default_open(true)
                        .show(ui, |ui| {
                            show_trees_desc("triggers:", triggers, context, ui, world);
                        });
                    CollapsingHeader::new("Targets")
                        .default_open(true)
                        .show(ui, |ui| {
                            show_trees_desc("targets:", targets, context, ui, world);
                        });

                    CollapsingHeader::new("Effects")
                        .default_open(true)
                        .show(ui, |ui| {
                            show_trees_desc("effects:", effects, context, ui, world);
                        });
                }
                Trigger::Change { .. } => todo!(),
                Trigger::List(_) => todo!(),
            }
        });

        let rep = &mut self.representation;
        rep.show_editor(context, "root", ui, world);
        ui.add_space(150.0);
    }
}

impl From<PackedUnit> for TableUnit {
    fn from(val: PackedUnit) -> Self {
        TableUnit {
            houses: val.houses,
            name: val.name,
            hp: val.hp,
            atk: val.atk,
            stacks: val.stacks,
            level: val.level,
            statuses: val
                .statuses
                .into_iter()
                .map(|(name, charges)| StatusCharges { name, charges })
                .collect_vec(),
            trigger: ron::to_string(&val.trigger).unwrap(),
            representation: ron::to_string(&val.representation).unwrap(),
            state: ron::to_string(&val.state).unwrap(),
        }
    }
}

impl From<TableUnit> for PackedUnit {
    fn from(value: TableUnit) -> Self {
        Self {
            hp: value.hp,
            atk: value.atk,
            stacks: value.stacks,
            level: value.level,
            houses: value.houses,
            name: value.name,
            trigger: ron::from_str(&value.trigger).unwrap(),
            representation: ron::from_str(&value.representation).unwrap(),
            state: ron::from_str(&value.state).unwrap(),
            statuses: value
                .statuses
                .into_iter()
                .map(|s| (s.name, s.charges))
                .collect_vec(),
        }
    }
}
