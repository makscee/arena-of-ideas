use bevy_egui::egui::{ComboBox, DragValue};

use crate::module_bindings::{StatusCharges, TableUnit};

use super::*;

#[derive(Deserialize, Serialize, TypeUuid, TypePath, Debug, Clone, PartialEq, Default)]
#[uuid = "028620be-3b01-4e20-b62e-a631f0db4777"]
#[serde(deny_unknown_fields)]
pub struct PackedUnit {
    pub hp: i32,
    pub atk: i32,
    #[serde(default = "default_one")]
    pub stacks: i32,
    #[serde(default = "default_one")]
    pub level: i32,
    #[serde(default = "default_house")]
    pub house: String,
    #[serde(default = "default_text")]
    pub name: String,
    #[serde(default)]
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

fn default_house() -> String {
    "Enemies".to_owned()
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
        let house_color = Pools::get(world)
            .houses
            .get(&self.house)
            .map(|h| h.color.clone())
            .unwrap_or_default();
        self.state
            .init(VarName::Hp, VarValue::Int(self.hp))
            .init(VarName::Atk, VarValue::Int(self.atk))
            .init(VarName::Stacks, VarValue::Int(self.stacks))
            .init(VarName::Level, VarValue::Int(self.level))
            .init(VarName::House, VarValue::String(self.house.clone()))
            .init(VarName::Name, VarValue::String(self.name.clone()))
            .init(VarName::Position, VarValue::Vec2(default()))
            .init(VarName::Index, VarValue::Int(0))
            .init(
                VarName::Slot,
                VarValue::Int(slot.unwrap_or_default() as i32),
            )
            .init(
                VarName::AbilityDescription,
                VarValue::String(self.description.to_owned()),
            )
            .init(
                VarName::TriggerDescription,
                VarValue::String(self.trigger.get_description_string()),
            )
            .init(
                VarName::HouseColor,
                VarValue::Color(house_color.clone().into()),
            )
            .init(VarName::Color, VarValue::Color(house_color.into()));
        self.state.clone().attach(entity, world);
        Status {
            name: LOCAL_TRIGGER.to_owned(),
            trigger: self.trigger.clone(),
        }
        .spawn(world)
        .insert(VarState::default())
        .set_parent(entity);
        for (status, charges) in self.statuses.iter() {
            if let Ok(entity) = Status::change_charges(status, entity, *charges, world) {
                Status::refresh_entity_mapping(entity, world);
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
        let description = state.get_string(VarName::AbilityDescription).unwrap();
        let house = state.get_string(VarName::House).unwrap();
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
            .clear_value(VarName::AbilityDescription)
            .clear_value(VarName::TriggerDescription)
            .clear_value(VarName::House)
            .simplify();

        Self {
            hp,
            atk,
            house,
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

    pub fn fuse(a: Self, b: Self) -> Vec<Trigger> {
        let mut result: Vec<Trigger> = default();
        let mut trigger_a = a.trigger;
        let mut trigger_b = b.trigger;

        if std::mem::discriminant(&trigger_a) != std::mem::discriminant(&trigger_b) {
            {
                if let Some(effect_a) = trigger_a.get_inner_effect().cloned() {
                    let mut trigger_b = trigger_b.clone();
                    trigger_b.set_inner_effect(effect_a);
                    result.push(Trigger::List(
                        [Box::new(trigger_a.clone()), Box::new(trigger_b)].into(),
                    ));
                }
                if let Some(effect_b) = trigger_b.get_inner_effect().cloned() {
                    let mut trigger_a = trigger_a.clone();
                    trigger_a.set_inner_effect(effect_b);
                    result.push(Trigger::List(
                        [Box::new(trigger_b.clone()), Box::new(trigger_a)].into(),
                    ));
                }
            }
        } else {
            let mut result_a = trigger_a.clone();
            let mut result_b = trigger_b.clone();
            if let Some(ability_a) = trigger_a.get_inner_effect().and_then(|e| e.find_ability()) {
                if let Some(ability_b) = trigger_b.get_inner_effect().and_then(|e| e.find_ability())
                {
                    *result_a.get_inner_effect().unwrap().find_ability().unwrap() = Effect::List(
                        [Box::new(ability_a.clone()), Box::new(ability_b.clone())].into(),
                    );
                    result.push(result_a);
                    *result_b.get_inner_effect().unwrap().find_ability().unwrap() = Effect::List(
                        [Box::new(ability_a.clone()), Box::new(ability_b.clone())].into(),
                    );
                    result.push(result_b);
                }
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
                        .selected_text(self.house.clone())
                        .show_ui(ui, |ui| {
                            for house in houses {
                                ui.selectable_value(&mut self.house, house.to_owned(), house);
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

impl Into<TableUnit> for PackedUnit {
    fn into(self) -> TableUnit {
        TableUnit {
            name: self.name,
            hp: self.hp,
            atk: self.atk,
            house: self.house,
            description: self.description,
            stacks: self.stacks,
            level: self.level,
            statuses: self
                .statuses
                .into_iter()
                .map(|(name, charges)| StatusCharges { name, charges })
                .collect_vec(),
            trigger: ron::to_string(&self.trigger).unwrap(),
            representation: ron::to_string(&self.representation).unwrap(),
            state: ron::to_string(&self.state).unwrap(),
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
            house: value.house,
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
