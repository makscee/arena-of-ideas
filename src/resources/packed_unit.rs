use strum_macros::FromRepr;

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
    pub pwr: i32,
    #[serde(default = "default_one")]
    pub stacks: i32,
    #[serde(default = "default_zero")]
    pub rarity: i32,
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
fn default_zero() -> i32 {
    0
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
            .init(VarName::Pwr, VarValue::Int(self.pwr))
            .init(VarName::Stacks, VarValue::Int(self.stacks))
            .init(
                VarName::Level,
                VarValue::Int(Self::level_from_stacks(self.stacks).0),
            )
            .init(VarName::Rarity, VarValue::Int(self.rarity))
            .init(
                VarName::RarityColor,
                VarValue::Color(Rarity::from_repr(self.rarity).unwrap().color().to_color()),
            )
            .init(VarName::Houses, VarValue::String(self.houses.clone()))
            .init(VarName::Name, VarValue::String(self.name.clone()))
            .init(VarName::Position, VarValue::Vec2(default()))
            .init(VarName::Index, VarValue::Int(0));
        if !state.has_value(VarName::Dmg) {
            state.init(VarName::Dmg, VarValue::Int(0));
        }
        self.trigger.inject_description(&mut state);
        let house_colors = self
            .houses
            .split('+')
            .map(|h| Pools::get_house_color(h, world).unwrap_or(Color::FUCHSIA))
            .collect_vec();
        state
            .init(VarName::Color, VarValue::Color(house_colors[0].clone()))
            .init(VarName::HouseColors, VarValue::ColorList(house_colors));
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

    /// 1 = 1
    /// 2 = 2
    /// 4 = 3
    /// 7 = 4
    pub fn level_from_stacks(stacks: i32) -> (i32, i32) {
        let lvl = (-1 + (1.0 + 8.0 * (stacks - 1) as f32).sqrt() as i32) / 2 + 1;
        let to_next = lvl * (lvl + 1) / 2 + 1 - stacks;
        (lvl, to_next)
    }

    pub fn pack(entity: Entity, world: &World) -> Self {
        let representation = Representation::pack(entity, world);
        let mut state = VarState::get(entity, world).clone();
        let hp = state.get_int(VarName::Hp).unwrap();
        let pwr = state.get_int(VarName::Pwr).unwrap();
        let stacks = state.get_int(VarName::Stacks).unwrap();
        let rarity = state.get_int(VarName::Rarity).unwrap();
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
            .clear_value(VarName::Pwr)
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
            pwr,
            houses,
            name,
            trigger,
            representation,
            state,
            statuses,
            stacks,
            rarity,
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
        fused.pwr = fused.pwr.max(source.pwr);
        fused.houses = format!("{}+{}", target.houses, source.houses);
        fused.name = format!("{}+{}", fused.name, source.name);
        fused.stacks = target.stacks + source.stacks - 3;
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
            let pwr = &mut self.pwr;
            ui.label("pwr:");
            DragValue::new(pwr).clamp_range(0..=99).ui(ui);
            let hp = &mut self.hp;
            ui.label("hp:");
            DragValue::new(hp).clamp_range(0..=99).ui(ui);
            let dmg = &mut self.state.get_int(VarName::Dmg).unwrap_or_default();
            ui.label("dmg:");
            if DragValue::new(dmg).clamp_range(0..=99).ui(ui).changed() {
                self.state.init(VarName::Dmg, VarValue::Int(*dmg));
            }
            let stacks = &mut self.stacks;
            ui.label("stacks:");
            DragValue::new(stacks).clamp_range(1..=99).ui(ui);
            let rarity = &mut self.rarity;
            ui.label("rarity:");
            DragValue::new(rarity).clamp_range(0..=3).ui(ui);
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

#[derive(FromRepr, AsRefStr, EnumIter, PartialEq, Clone, Copy)]
#[repr(i32)]
pub enum Rarity {
    Common,
    Rare,
    Epic,
    Legendary,
}

impl Rarity {
    pub fn color(&self) -> Color32 {
        match self {
            Rarity::Common => hex_color!("#B0BEC5"),
            Rarity::Rare => hex_color!("#0277BD"),
            Rarity::Epic => hex_color!("#AB47BC"),
            Rarity::Legendary => hex_color!("#F57C00"),
        }
    }
}

impl From<PackedUnit> for TableUnit {
    fn from(val: PackedUnit) -> Self {
        TableUnit {
            houses: val.houses,
            name: val.name,
            hp: val.hp,
            pwr: val.pwr,
            stacks: val.stacks,
            rarity: val.rarity,
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
            pwr: value.pwr,
            stacks: value.stacks,
            rarity: value.rarity,
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
