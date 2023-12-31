use bevy_egui::egui::{ComboBox, DragValue};

use super::*;

#[derive(Deserialize, Serialize, TypeUuid, TypePath, Debug, Clone, PartialEq, Default)]
#[uuid = "028620be-3b01-4e20-b62e-a631f0db4777"]
#[serde(deny_unknown_fields)]
pub struct PackedUnit {
    pub hp: i32,
    pub atk: i32,
    #[serde(default = "default_stacks")]
    pub stacks: i32,
    #[serde(default = "default_house")]
    pub house: String,
    #[serde(default)]
    pub trigger: Trigger,
    #[serde(default = "default_text")]
    pub name: String,
    #[serde(default)]
    pub description: String,
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
fn default_stacks() -> i32 {
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
            .init(VarName::House, VarValue::String(self.house.clone()))
            .init(VarName::Name, VarValue::String(self.name.clone()))
            .init(VarName::Position, VarValue::Vec2(default()))
            .init(VarName::Index, VarValue::Int(0))
            .init(
                VarName::Slot,
                VarValue::Int(slot.unwrap_or_default() as i32),
            )
            .init(
                VarName::Description,
                VarValue::String(self.description.to_owned()),
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
        let card = UnitCard::from_entity(entity, world)
            .unwrap()
            .set_open(false);
        world
            .entity_mut(entity)
            .insert((Name::new(self.name.clone()), Unit))
            .insert(card);
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
        let state = VarState::get(entity, world).clone();
        let hp = state.get_int(VarName::Hp).unwrap();
        let atk = state.get_int(VarName::Atk).unwrap();
        let stacks = state.get_int(VarName::Stacks).unwrap();
        let name = state.get_string(VarName::Name).unwrap();
        let description = state.get_string(VarName::Description).unwrap();
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
        }
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
