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
    ) -> i32 {
        match self {
            ExpressionInt::Sum { a, b } => {
                a.calculate(context, world, resources) + b.calculate(context, world, resources)
            }
            ExpressionInt::Mul { a, b } => {
                a.calculate(context, world, resources) * b.calculate(context, world, resources)
            }
            ExpressionInt::Const { value } => *value,
            ExpressionInt::Var { var } => context.vars.get_int(var),
            ExpressionInt::Stat { stat, target } => {
                let target = target
                    .as_ref()
                    .and_then(|target| Some(target.calculate(&context, world, resources)))
                    .unwrap_or(context.target);
                let target = world.entry_ref(target).unwrap();
                match stat {
                    StatType::Hp => target.get_component::<HpComponent>().unwrap().current,
                    StatType::Attack => target.get_component::<AttackComponent>().unwrap().value,
                }
            }
            ExpressionInt::AbilityVar {
                house,
                ability: ability_name,
                var: var_name,
            } => resources
                .houses
                .get(house)
                .expect(&format!("Failed to get {:?}", house))
                .abilities
                .get(ability_name)
                .expect(&format!(
                    "Failed to get Ability {} from {:?}",
                    ability_name, house
                ))
                .vars
                .get_int(var_name),
            ExpressionInt::StatusVar { status, var } => resources
                .status_pool
                .active_statuses
                .get(&context.target)
                .expect(&format!("Failed to get target#{:?}", context.target))
                .get(status)
                .expect(&format!(
                    "Failed to find status {} on {:?}",
                    status, context.target
                ))
                .vars
                .get_int(var),
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
    ) -> legion::Entity {
        match self {
            ExpressionEntity::World => {
                <(&WorldComponent, &EntityComponent)>::query()
                    .iter(world)
                    .next()
                    .unwrap()
                    .1
                    .entity
            }
            ExpressionEntity::Target => context.target,
            ExpressionEntity::Parent => context.parent.expect("Failed to get parent"),
            ExpressionEntity::Owner => context.owner,
            ExpressionEntity::FindUnit { slot, faction } => {
                let slot = slot.calculate(context, world, resources) as usize;
                WorldSystem::collect_factions(world, &hashset! {*faction})
                    .into_iter()
                    .find_map(|(entity, unit)| match unit.slot == slot {
                        true => Some(entity),
                        false => None,
                    })
                    .expect(&format!("No unit of {:?} found in {} slot", faction, slot))
            }
            ExpressionEntity::RandomUnit { faction } => {
                <(&UnitComponent, &EntityComponent)>::query()
                    .iter(world)
                    .filter_map(|(unit, entity)| match unit.faction == *faction {
                        true => Some(entity.entity),
                        false => None,
                    })
                    .choose(&mut thread_rng())
                    .expect(&format!("No units of {:?} found", faction))
            }
        }
    }
}
