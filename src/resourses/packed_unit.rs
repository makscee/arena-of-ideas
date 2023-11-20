use bevy_egui::egui::{ComboBox, DragValue};

use super::*;

#[derive(Deserialize, Serialize, TypeUuid, TypePath, Debug, Clone, PartialEq, Default)]
#[uuid = "028620be-3b01-4e20-b62e-a631f0db4777"]
#[serde(deny_unknown_fields)]
pub struct PackedUnit {
    pub hp: i32,
    pub atk: i32,
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

const LOCAL_TRIGGER: &str = "_local";

impl PackedUnit {
    pub fn unpack(mut self, parent: Entity, slot: Option<usize>, world: &mut World) -> Entity {
        debug!("Unpack unit {:?}", &self);
        let entity = Options::get_unit_rep(world)
            .clone()
            .unpack(None, Some(parent), world);
        world
            .entity_mut(entity)
            .insert(PickableBundle::default())
            .insert(RaycastPickTarget::default())
            .insert(On::<Pointer<Over>>::run(UnitPlugin::hover_unit))
            .insert(On::<Pointer<Out>>::run(UnitPlugin::unhover_unit))
            .insert(On::<Pointer<DragStart>>::run(UnitPlugin::drag_unit_start))
            .insert(On::<Pointer<DragEnd>>::run(UnitPlugin::drag_unit_end))
            .insert(On::<Pointer<Drag>>::run(UnitPlugin::drag_unit));
        {
            let entity = self.representation.unpack(None, Some(entity), world);
            world.entity_mut(entity).insert(UnitRepresentation);
        }
        let house_color = Pools::get(world)
            .houses
            .get(&self.house)
            .and_then(|h| Some(h.color.clone()))
            .unwrap_or_default();
        self.state
            .init(VarName::Hp, VarValue::Int(self.hp))
            .init(VarName::Atk, VarValue::Int(self.atk))
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
        world
            .entity_mut(entity)
            .insert((Unit, Name::new(self.name)));
        self.state.attach(entity, world);
        Status::spawn(LOCAL_TRIGGER.to_owned(), self.trigger, world)
            .insert(VarState::default())
            .set_parent(entity);
        for (status, charges) in self.statuses {
            if let Ok(entity) = Status::change_charges(&status, entity, charges, world) {
                Status::apply_delta(entity, world);
            }
        }
        entity
    }

    pub fn pack(entity: Entity, world: &World) -> Self {
        let rep_entity = world
            .get::<Children>(entity)
            .unwrap()
            .into_iter()
            .find(|x| world.get::<UnitRepresentation>(**x).is_some());
        let representation = {
            match rep_entity {
                Some(rep_entity) => Representation::pack(*rep_entity, world),
                None => default(),
            }
        };
        let state = VarState::get(entity, world).clone();
        let hp = state.get_int(VarName::Hp).unwrap();
        let atk = state.get_int(VarName::Atk).unwrap();
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
                    ui.label("hp:");
                    ui.add(DragValue::new(&mut self.hp));
                    ui.label("atk:");
                    ui.add(DragValue::new(&mut self.atk));
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
}
