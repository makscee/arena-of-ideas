use bevy_egui::egui::{ComboBox, DragValue};

use crate::module_bindings::{StatusCharges, TableUnit};

use super::*;

#[derive(Deserialize, Serialize, TypeUuid, TypePath, Debug, Clone, PartialEq, Default)]
#[uuid = "028620be-3b01-4e20-b62e-a631f0db4777"]
#[serde(deny_unknown_fields)]
pub struct PackedUnit {
    #[serde(default = "default_text")]
    pub name: String,
    #[serde(default = "default_one")]
    pub hp: i32,
    pub atk: i32,
    #[serde(default = "default_one")]
    pub stacks: i32,
    #[serde(default = "default_one")]
    pub level: i32,
    #[serde(default = "default_houses")]
    pub houses: String,
    #[serde(default = "default_description ")]
    pub description: String,
    #[serde(default)]
    pub trigger: Trigger,
    #[serde(default)]
    pub representation: Representation,
    #[serde(default)]
    pub state: VarState,
    #[serde(default)]
    pub statuses: Vec<(String, i32)>,
}

fn default_description() -> String {
    "%trigger → %effect on %target".to_owned()
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
        let entity = if SkipVisual::active(world) {
            world.spawn_empty().set_parent(parent).id()
        } else {
            Options::get_unit_rep(world)
                .clone()
                .unpack(None, Some(parent), world)
        };
        let is_team = VarState::get(parent, world)
            .get_faction(VarName::Faction)
            .unwrap()
            .eq(&Faction::Team);
        let mut emut = world.entity_mut(entity);
        emut.insert(PickableBundle::default())
            .insert(RaycastPickTarget::default())
            .insert(On::<Pointer<Over>>::run(UnitPlugin::hover_unit))
            .insert(On::<Pointer<Out>>::run(UnitPlugin::unhover_unit));
        if is_team {
            emut.insert(On::<Pointer<DragStart>>::run(UnitPlugin::drag_unit_start))
                .insert(On::<Pointer<DragEnd>>::run(UnitPlugin::drag_unit_end))
                .insert(On::<Pointer<Drag>>::run(UnitPlugin::drag_unit));
        }
        if !SkipVisual::active(world) {
            let entity = self
                .representation
                .clone()
                .unpack(None, Some(entity), world);
            world.entity_mut(entity).insert(UnitRepresentation);
        }

        self.state = self.generate_state(world);
        self.state.init(
            VarName::Slot,
            VarValue::Int(slot.unwrap_or_default() as i32),
        );
        self.state.clone().attach(entity, world);
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
            let _ = Status::change_charges(status, entity, *charges, world);
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
        let description = self.description.clone();
        state
            .init(VarName::Hp, VarValue::Int(self.hp))
            .init(VarName::Atk, VarValue::Int(self.atk))
            .init(VarName::Stacks, VarValue::Int(self.stacks))
            .init(VarName::Level, VarValue::Int(self.level))
            .init(VarName::Houses, VarValue::String(self.houses.clone()))
            .init(VarName::Name, VarValue::String(self.name.clone()))
            .init(VarName::Position, VarValue::Vec2(default()))
            .init(VarName::Index, VarValue::Int(0))
            .init(
                VarName::Description,
                VarValue::String(self.description.clone()),
            )
            .init(VarName::Description, VarValue::String(description));
        self.trigger.inject_description(&mut state);
        let house_colors = self
            .houses
            .split('+')
            .map(|h| Pools::get_house_color(h, world).unwrap_or(Color::CRIMSON))
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
            .find(|x| world.get::<UnitRepresentation>(**x).is_some())
            .copied()
    }

    pub fn pack(entity: Entity, world: &World) -> Self {
        let representation = {
            match Self::get_representation_entity(entity, world) {
                Some(entity) => Representation::pack(entity, world),
                None => default(),
            }
        };
        let mut state = VarState::get(entity, world).clone();
        let hp = state.get_int(VarName::Hp).unwrap();
        let atk = state.get_int(VarName::Atk).unwrap();
        let stacks = state.get_int(VarName::Stacks).unwrap();
        let level = state.get_int(VarName::Level).unwrap();
        let name = state.get_string(VarName::Name).unwrap();
        let description = state.get_string(VarName::Description).unwrap();
        let houses = state.get_string(VarName::Houses).unwrap();
        let mut trigger = None;
        let mut statuses = Vec::default();
        for entity in Status::collect_entity_statuses(entity, world) {
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
            description,
            statuses,
            stacks,
            level,
        }
    }

    fn fuse_base(a: &Self, b: &Self, trigger: Trigger, world: &World) -> Self {
        let mut fused = a.clone();
        fused.representation.children.push(Box::new({
            let mut rep = b.representation.clone();
            rep.mapping.insert(
                VarName::Color,
                Expression::Value(VarValue::Color(
                    Pools::get_house_color(b.houses.split('+').next().unwrap(), world).unwrap(),
                )),
            );
            rep
        }));
        fused.hp = fused.hp.max(b.hp);
        fused.atk = fused.atk.max(b.atk);
        fused.houses = format!("{}+{}", a.houses, b.houses);
        fused.name = format!("{}+{}", fused.name, b.name);
        fused.level = 1;
        fused.stacks = 1;
        fused.trigger = trigger;
        fused
    }

    pub fn fuse(a: Self, b: Self, world: &World) -> Vec<Self> {
        let mut result: Vec<Self> = default();
        let trigger_a = &a.trigger;
        let trigger_b = &b.trigger;

        match (trigger_a, trigger_b) {
            (
                Trigger::Fire {
                    trigger: trigger_a,
                    target: target_a,
                    effect: effect_a,
                },
                Trigger::Fire {
                    trigger: trigger_b,
                    target: target_b,
                    effect: effect_b,
                },
            ) => {
                if !trigger_a.eq(trigger_b) {
                    let trigger = Trigger::Fire {
                        trigger: FireTrigger::List(
                            [Box::new(trigger_a.clone()), Box::new(trigger_b.clone())].into(),
                        ),
                        target: target_a.clone(),
                        effect: effect_a.clone(),
                    };
                    result.push(Self::fuse_base(&a, &b, trigger, world));
                    let trigger = Trigger::Fire {
                        trigger: FireTrigger::List(
                            [Box::new(trigger_a.clone()), Box::new(trigger_b.clone())].into(),
                        ),
                        target: target_b.clone(),
                        effect: effect_b.clone(),
                    };
                    result.push(Self::fuse_base(&b, &a, trigger, world));
                }
                let trigger = Trigger::Fire {
                    trigger: trigger_a.clone(),
                    target: target_a.clone(),
                    effect: Effect::List(
                        [Box::new(effect_a.clone()), Box::new(effect_b.clone())].into(),
                    ),
                };
                result.push(Self::fuse_base(&a, &b, trigger, world));
                let trigger = Trigger::Fire {
                    trigger: trigger_b.clone(),
                    target: target_b.clone(),
                    effect: Effect::List(
                        [Box::new(effect_a.clone()), Box::new(effect_b.clone())].into(),
                    ),
                };
                result.push(Self::fuse_base(&b, &a, trigger, world));
            }
            _ => {
                let trigger = Trigger::List(
                    [Box::new(trigger_a.clone()), Box::new(trigger_b.clone())].into(),
                );
                result.push(Self::fuse_base(&a, &b, trigger, world));
            }
        }

        result
    }

    pub fn show_editor(
        &mut self,
        entity: Option<Entity>,
        editing_data: &mut EditingData,
        ui: &mut Ui,
        world: &mut World,
    ) {
        CollapsingHeader::new("Hero")
            .default_open(true)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.add(
                        TextEdit::singleline(&mut self.name)
                            .desired_width(80.0)
                            .hint_text("name"),
                    );
                    ui.add(
                        TextEdit::singleline(&mut self.description)
                            .desired_width(100.0)
                            .hint_text("description"),
                    );
                    ui.label("atk:");
                    ui.add(DragValue::new(&mut self.atk));
                    ui.label("hp:");
                    ui.add(DragValue::new(&mut self.hp));
                    let houses = Pools::get(world).houses.keys().cloned().collect_vec();
                    ui.label("house:");
                    ComboBox::from_id_source("house")
                        .selected_text(self.houses.clone())
                        .show_ui(ui, |ui| {
                            for house in houses {
                                ui.selectable_value(&mut self.houses, house.clone(), house);
                            }
                        })
                });

                self.representation
                    .show_editor(entity, editing_data, 0, ui, world);
                self.trigger
                    .show_editor(editing_data, "trigger".to_owned(), ui, world);
            });
    }

    pub fn statuses_string(&self) -> String {
        self.statuses
            .iter()
            .map(|(name, charges)| format!("{name} ({charges})"))
            .join(",")
    }
}

impl From<PackedUnit> for TableUnit {
    fn from(val: PackedUnit) -> Self {
        TableUnit {
            houses: val.houses,
            name: val.name,
            hp: val.hp,
            atk: val.atk,
            description: val.description,
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
            description: value.description,
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
