use ron::to_string;

use super::*;

#[derive(Asset, Deserialize, Serialize, TypePath, Debug, Clone, PartialEq, Default)]
#[serde(deny_unknown_fields)]
pub struct PackedUnit {
    #[serde(default = "default_empty")]
    pub name: String,
    #[serde(default = "default_one")]
    pub pwr: i32,
    #[serde(default = "default_one")]
    pub hp: i32,
    #[serde(default = "default_zero_i8")]
    pub rarity: i8,
    #[serde(default = "default_house")]
    pub houses: Vec<String>,
    #[serde(default)]
    pub representation: Representation,
    #[serde(default)]
    pub trigger: Trigger,
    #[serde(default)]
    pub state: VarState,
    #[serde(default)]
    pub statuses: Vec<(String, i32)>,
}

fn default_one() -> i32 {
    1
}
fn default_zero_i8() -> i8 {
    0
}
fn default_empty() -> String {
    "_empty".to_owned()
}
fn default_house() -> Vec<String> {
    vec!["Default".to_owned()]
}

impl PackedUnit {
    pub fn unpack(mut self, parent: Entity, slot: Option<i32>, world: &mut World) -> Entity {
        let entity = world
            .spawn_empty()
            .set_parent(parent)
            .insert((
                Unit,
                VisibilityBundle::default(),
                TransformBundle::default(),
                TextColumn::default(),
            ))
            .id();
        debug!("unpack unit: {entity:?} {self:?}");
        self.state = self.generate_state(world);
        {
            self.representation.unpack(entity, world);
            let entity = GameAssets::get(world)
                .unit_rep
                .clone()
                .unpack(world.spawn_empty().set_parent(entity).id(), world);
            let mut emut = world.entity_mut(entity);
            emut.get_mut::<Transform>().unwrap().translation.z += 100.0;
        }
        self.state
            .init(VarName::Slot, VarValue::Int(slot.unwrap_or_default()));
        Status {
            name: "_local".to_owned(),
            trigger: self.trigger,
        }
        .spawn(entity, world);
        self.state.add_status(
            "_local".to_owned(),
            VarState::default()
                .init(VarName::Charges, VarValue::Int(1))
                .take(),
        );
        self.state.attach(entity, world);
        for (status, charges) in self.statuses {
            let _ = Status::change_charges(&status, entity, charges, world);
        }
        Status::refresh_mappings(entity, world);
        entity
    }

    pub fn generate_state(&self, world: &World) -> VarState {
        let mut state = self.state.clone();
        state
            .init(VarName::Hp, self.hp.into())
            .init(VarName::Pwr, self.pwr.into())
            .init(VarName::Name, self.name.clone().into())
            .init(VarName::Position, Vec2::ZERO.into())
            .init(VarName::Visible, true.into());
        if !state.has_value(VarName::Dmg) {
            state.init(VarName::Dmg, VarValue::Int(0));
        }
        state
    }
}

impl From<PackedUnit> for BaseUnit {
    fn from(value: PackedUnit) -> Self {
        let (triggers, targets, effects) = match value.trigger {
            Trigger::Fire {
                triggers,
                targets,
                effects,
            } => (
                triggers
                    .into_iter()
                    .map(|t| to_string(&t).unwrap())
                    .collect_vec(),
                targets
                    .into_iter()
                    .map(|t| to_string(&t).unwrap())
                    .collect_vec(),
                effects
                    .into_iter()
                    .map(|t| to_string(&t).unwrap())
                    .collect_vec(),
            ),
            _ => (default(), default(), default()),
        };
        Self {
            name: value.name,
            pwr: value.pwr,
            hp: value.hp,
            rarity: value.rarity,
            house: value.houses.first().unwrap().clone(),
            repr: default(),
            triggers,
            targets,
            effects,
        }
    }
}

impl From<BaseUnit> for PackedUnit {
    fn from(value: BaseUnit) -> Self {
        let triggers = value
            .triggers
            .into_iter()
            .map(|t| ron::from_str::<(FireTrigger, Option<String>)>(&t).unwrap())
            .collect_vec();
        let targets = value
            .targets
            .into_iter()
            .map(|t| ron::from_str::<(Expression, Option<String>)>(&t).unwrap())
            .collect_vec();
        let effects = value
            .effects
            .into_iter()
            .map(|t| ron::from_str::<(Effect, Option<String>)>(&t).unwrap())
            .collect_vec();
        Self {
            name: value.name,
            pwr: value.pwr,
            hp: value.hp,
            rarity: value.rarity,
            houses: vec![value.house],
            trigger: Trigger::Fire {
                triggers,
                targets,
                effects,
            },
            state: default(),
            statuses: default(),
            representation: default(),
        }
    }
}
