/// Expression is anything that can return a value.
/// For each return type there should be one enum
use super::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ExpressionInt {
    Sum {
        a: Box<ExpressionInt>,
        b: Box<ExpressionInt>,
    },
    Mul {
        a: Box<ExpressionInt>,
        b: Box<ExpressionInt>,
    },
    Const {
        value: i32,
    },
    Var {
        var: VarName,
    },
    EntityVar {
        var: VarName,
        entity: ExpressionEntity,
    },
    AbilityVar {
        house: HouseName,
        ability: String,
        var: VarName,
    },
}

impl ExpressionInt {
    pub fn calculate(
        &self,
        context: &Context,
        world: &legion::World,
        resources: &Resources,
    ) -> Result<i32, Error> {
        match self {
            ExpressionInt::Sum { a, b } => {
                Ok(a.calculate(context, world, resources)?
                    + b.calculate(context, world, resources)?)
            }
            ExpressionInt::Mul { a, b } => {
                Ok(a.calculate(context, world, resources)?
                    * b.calculate(context, world, resources)?)
            }
            ExpressionInt::Const { value } => Ok(*value),
            ExpressionInt::Var { var } => Ok(context.vars.get_int(var)),
            ExpressionInt::EntityVar { var, entity } => {
                ContextSystem::try_get_context(entity.calculate(context, world, resources)?, world)?
                    .vars
                    .try_get_int(var)
                    .context(format!("Var not found {}", var))
            }
            ExpressionInt::AbilityVar {
                house,
                ability,
                var,
            } => {
                let faction = Faction::from_entity(context.owner, world, &resources);
                Ok(TeamPool::get_ability_var_int(
                    house, ability, var, &faction, resources,
                ))
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ExpressionEntity {
    World,
    Target,
    Parent,
    Owner,
    FindUnit {
        slot: Box<ExpressionInt>,
        faction: Faction,
    },
    RandomUnit {
        faction: Faction,
    },
}

impl ExpressionEntity {
    pub fn calculate(
        &self,
        context: &Context,
        world: &legion::World,
        resources: &Resources,
    ) -> Result<legion::Entity, Error> {
        match self {
            ExpressionEntity::World => Ok(<(&WorldComponent, &EntityComponent)>::query()
                .iter(world)
                .next()
                .unwrap()
                .1
                .entity),
            ExpressionEntity::Target => Ok(context.target),
            ExpressionEntity::Parent => context.parent.context("Failed to get parent"),
            ExpressionEntity::Owner => Ok(context.owner),
            ExpressionEntity::FindUnit { slot, faction } => {
                let slot = slot.calculate(context, world, resources)? as usize;
                UnitSystem::collect_factions(world, &hashset! {*faction})
                    .into_iter()
                    .find_map(|(entity, unit)| match unit.slot == slot {
                        true => Some(entity),
                        false => None,
                    })
                    .context(format!("No unit of {:?} found in {} slot", faction, slot))
            }
            ExpressionEntity::RandomUnit { faction } => {
                <(&UnitComponent, &EntityComponent)>::query()
                    .iter(world)
                    .filter_map(|(unit, entity)| match unit.faction == *faction {
                        true => Some(entity.entity),
                        false => None,
                    })
                    .choose(&mut thread_rng())
                    .context(format!("No units of {:?} found", faction))
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ExpressionFaction {
    Owner,
    Target,
    Parent,
}

impl ExpressionFaction {
    pub fn calculate(
        &self,
        context: &Context,
        world: &legion::World,
        _: &Resources,
    ) -> Result<Faction, Error> {
        let entity = match self {
            ExpressionFaction::Owner => context.owner,
            ExpressionFaction::Target => context.target,
            ExpressionFaction::Parent => context.parent.context("No parent")?,
        };
        Ok(world
            .entry_ref(entity)
            .context("Failed to find entity")?
            .get_component::<UnitComponent>()?
            .faction)
    }
}
