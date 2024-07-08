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
    pub fn unpack(
        mut self,
        parent: Entity,
        slot: Option<i32>,
        id: Option<u64>,
        world: &mut World,
    ) -> Entity {
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
        debug!("unpack unit: #{id:?} {entity:?} {self:?}");
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
            .init(VarName::Slot, slot.unwrap_or_default().into());
        Status {
            name: "_local".to_owned(),
            trigger: self.trigger,
        }
        .spawn(entity, world);
        self.state.add_status(
            "_local".to_owned(),
            VarState::default().init(VarName::Charges, 1.into()).take(),
        );
        self.state.attach(entity, id.unwrap_or_default(), world);
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
            .init(VarName::Lvl, 1.into())
            .init(VarName::Stacks, 1.into())
            .init(VarName::Name, self.name.clone().into())
            .init(
                VarName::Houses,
                VarValue::List(
                    self.houses
                        .clone()
                        .into_iter()
                        .map(|s| s.into())
                        .collect_vec(),
                ),
            )
            .init(VarName::Position, Vec2::ZERO.into())
            .init(VarName::Visible, true.into())
            .init(
                VarName::RarityColor,
                rarity_color(self.rarity as usize).into(),
            );
        if let Some(house) = self.houses.iter().next() {
            state.init(VarName::Color, name_color(house).into());
        }
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
            triggers,
            targets,
            effects,
        }
    }
}

impl From<BaseUnit> for PackedUnit {
    fn from(value: BaseUnit) -> Self {
        let triggers = value.triggers();
        let targets = value.targets();
        let effects = value.effects();
        let representation =
            RepresentationPlugin::get_by_id(value.name.clone()).unwrap_or_default();
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
            representation,
            state: default(),
            statuses: default(),
        }
    }
}

impl From<FusedUnit> for PackedUnit {
    fn from(value: FusedUnit) -> Self {
        let bases: Vec<BaseUnit> = value
            .bases
            .iter()
            .map(|name| {
                BaseUnit::filter_by_name(name.clone())
                    .with_context(|| format!("BaseUnit {name} not found"))
                    .unwrap()
            })
            .collect_vec();
        let mut result = Self::default();
        let triggers = value
            .triggers
            .into_iter()
            .flat_map(|i| bases[i as usize].triggers())
            .collect_vec();
        let targets = value
            .targets
            .into_iter()
            .flat_map(|i| bases[i as usize].targets())
            .collect_vec();
        let effects = value
            .effects
            .into_iter()
            .flat_map(|i| bases[i as usize].effects())
            .collect_vec();
        result.trigger = Trigger::Fire {
            triggers,
            targets,
            effects,
        };
        for base in bases {
            result.pwr = result.pwr.max(base.pwr);
            result.hp = result.hp.max(base.hp);
            result.rarity = result.rarity.max(base.rarity);
            result.houses.push(base.house);
            if let Some(repr) = RepresentationPlugin::get_by_id(base.name.clone()) {
                result.representation.children.push(Box::new(repr));
            }
        }
        result.name = value.bases.join("+");

        result
    }
}

trait BaseUnitExtract {
    fn triggers(&self) -> Vec<(FireTrigger, Option<String>)>;
    fn targets(&self) -> Vec<(Expression, Option<String>)>;
    fn effects(&self) -> Vec<(Effect, Option<String>)>;
}

impl BaseUnitExtract for BaseUnit {
    fn triggers(&self) -> Vec<(FireTrigger, Option<String>)> {
        self.triggers
            .iter()
            .map(|t| ron::from_str::<(FireTrigger, Option<String>)>(t).unwrap())
            .collect_vec()
    }
    fn targets(&self) -> Vec<(Expression, Option<String>)> {
        self.targets
            .iter()
            .map(|t| ron::from_str::<(Expression, Option<String>)>(t).unwrap())
            .collect_vec()
    }
    fn effects(&self) -> Vec<(Effect, Option<String>)> {
        self.effects
            .iter()
            .map(|t| ron::from_str::<(Effect, Option<String>)>(t).unwrap())
            .collect_vec()
    }
}
