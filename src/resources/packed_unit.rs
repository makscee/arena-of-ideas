use ron::to_string;

use super::*;

#[derive(Asset, Deserialize, Serialize, TypePath, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PackedUnit {
    #[serde(default = "default_empty")]
    pub name: String,
    #[serde(default = "default_one")]
    pub pwr: i32,
    #[serde(default = "default_one")]
    pub hp: i32,
    #[serde(default = "default_zero")]
    pub pwr_mutation: i32,
    #[serde(default = "default_zero")]
    pub hp_mutation: i32,
    #[serde(default = "default_one")]
    pub lvl: i32,
    #[serde(default = "default_zero")]
    pub xp: i32,
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
fn default_zero() -> i32 {
    0
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

pub const LOCAL_STATUS: &str = "_local";

impl Default for PackedUnit {
    fn default() -> Self {
        ron::from_str("(pwr: 1, hp: 3)").unwrap()
    }
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
        debug!("unpack unit: #{id:?} {entity} {self:?}");
        save_entity_name(entity, UnitPlugin::name_cstr(&self.name));
        self.state = self.generate_state();
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
            name: LOCAL_STATUS.to_owned(),
            trigger: self.trigger,
        }
        .spawn(entity, world);
        self.state.add_status(
            LOCAL_STATUS.to_owned(),
            VarState::default().init(VarName::Charges, 1.into()).take(),
        );
        self.state.attach(entity, id.unwrap_or_default(), world);
        for (status, charges) in self.statuses {
            let _ = Status::change_charges(&status, entity, charges, world);
        }
        Status::refresh_mappings(entity, world);
        entity
    }

    pub fn generate_state(&self) -> VarState {
        let mut state = self.state.clone();

        state
            .init(VarName::Hp, self.hp.into())
            .init(VarName::Pwr, self.pwr.into())
            .init(VarName::HpMutation, self.hp_mutation.into())
            .init(VarName::PwrMutation, self.pwr_mutation.into())
            .init(VarName::Lvl, self.lvl.into())
            .init(VarName::Xp, self.xp.into())
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
            .init(VarName::RarityColor, rarity_color(self.rarity).into());
        if !state.has_value(VarName::HouseColors) {
            state.init(
                VarName::HouseColors,
                self.houses
                    .iter()
                    .map(|h| name_color(h))
                    .collect_vec()
                    .into(),
            );
            state.init(
                VarName::RarityColors,
                vec![rarity_color(self.rarity)].into(),
            );
        }
        let (triggers, targets, effects) = self.trigger.parse_fire_strings();
        let mut used_definitions: HashSet<String> = default();
        let full_description = triggers
            .iter()
            .chain(targets.iter())
            .chain(effects.iter())
            .map(|c| c.to_string())
            .join(" ");
        for name in definition_names() {
            if full_description.contains(&name) {
                used_definitions.insert(name);
            }
        }
        state
            .init(VarName::TriggersDescription, triggers.into())
            .init(VarName::TargetsDescription, targets.into())
            .init(VarName::EffectsDescription, effects.into())
            .init(
                VarName::UsedDefinitions,
                used_definitions.into_iter().collect_vec().into(),
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

impl From<PackedUnit> for TBaseUnit {
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

impl From<TBaseUnit> for PackedUnit {
    fn from(value: TBaseUnit) -> Self {
        let triggers = value.triggers();
        let targets = value.targets();
        let effects = value.effects();
        let representation =
            RepresentationPlugin::get_by_id(value.name.clone()).unwrap_or_default();
        let mut state = VarState::default();
        state.init(VarName::HouseColors, vec![name_color(&value.house)].into());
        state.init(
            VarName::RarityColors,
            vec![rarity_color(value.rarity)].into(),
        );
        Self {
            name: value.name,
            pwr: value.pwr,
            hp: value.hp,
            pwr_mutation: 0,
            hp_mutation: 0,
            lvl: 1,
            xp: 0,
            rarity: value.rarity,
            houses: vec![value.house],
            trigger: Trigger::Fire {
                triggers,
                targets,
                effects,
            },
            representation,
            state,
            statuses: default(),
        }
    }
}

impl From<FusedUnit> for PackedUnit {
    fn from(value: FusedUnit) -> Self {
        let bases: Vec<TBaseUnit> = value
            .bases
            .iter()
            .map(|name| name.base_unit())
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
        let mut state = VarState::default();
        let mut rarity_colors: Vec<Color> = default();
        let mut house_colors: Vec<Color> = default();
        for base in bases {
            let house_color = name_color(&base.house).to_color();
            rarity_colors.push(rarity_color(base.rarity).to_color());
            house_colors.push(house_color);
            result.rarity = result.rarity.max(base.rarity);
            result.houses.push(base.house);
            if let Some(mut repr) = RepresentationPlugin::get_by_id(base.name.clone()) {
                repr.mapping
                    .insert(VarName::Color, Expression::Value(house_color.into()));
                result.representation.children.push(Box::new(repr));
            }
        }
        result.pwr = value.pwr;
        result.hp = value.hp;
        result.pwr_mutation = value.pwr_mutation;
        result.hp_mutation = value.hp_mutation;
        state.init(VarName::RarityColors, rarity_colors.into());
        state.init(VarName::HouseColors, house_colors.into());
        result.name = value.bases.join("+");
        result.xp = value.xp as i32;
        result.lvl = value.lvl as i32;
        result.state = state;

        result
    }
}

trait BaseUnitExtract {
    fn triggers(&self) -> Vec<(FireTrigger, Option<String>)>;
    fn targets(&self) -> Vec<(Expression, Option<String>)>;
    fn effects(&self) -> Vec<(Effect, Option<String>)>;
}

impl BaseUnitExtract for TBaseUnit {
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
