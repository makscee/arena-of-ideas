/// Expression is anything that can return a value.
/// For each return type there should be one enum
use legion::EntityStore;

use super::*;

#[derive(Clone, Debug, Deserialize)]
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
    AbilityVar {
        house: HouseName,
        ability: String,
        var: VarName,
    },
    StatusVar {
        status: String,
        var: VarName,
    },
    Stat {
        stat: StatType,
        target: Option<ExpressionEntity>,
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
            ExpressionInt::Stat { stat, target } => {
                let target = target
                    .as_ref()
                    .and_then(|target| Some(target.calculate(&context, world, resources)))
                    .unwrap_or(Ok(context.target));
                let target = world.entry_ref(target?).unwrap();
                Ok(match stat {
                    StatType::Hp => target.get_component::<HpComponent>().unwrap().current,
                    StatType::Attack => target.get_component::<AttackComponent>().unwrap().value,
                })
            }
            ExpressionInt::AbilityVar {
                house,
                ability: ability_name,
                var: var_name,
            } => Ok(resources
                .houses
                .get(house)
                .context(format!("Failed to get {:?}", house))?
                .abilities
                .get(ability_name)
                .context(format!(
                    "Failed to get Ability {} from {:?}",
                    ability_name, house
                ))?
                .vars
                .get_int(var_name)),
            ExpressionInt::StatusVar { status, var } => Ok(resources
                .status_pool
                .active_statuses
                .get(&context.target)
                .context(format!("Failed to get target#{:?}", context.target))?
                .get(status)
                .context(format!(
                    "Failed to find status {} on {:?}",
                    status, context.target
                ))?
                .vars
                .get_int(var)),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
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
