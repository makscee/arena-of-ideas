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
    Const {
        value: i32,
    },
    Var {
        name: VarName,
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
            ExpressionInt::Const { value } => *value,
            ExpressionInt::Var { name } => context.vars.get_int(name),
            ExpressionInt::Stat { stat } => {
                let target = world.entry_ref(context.target).unwrap();
                match stat {
                    StatType::Hp => target.get_component::<HpComponent>().unwrap().current(),
                    StatType::Attack => target.get_component::<AttackComponent>().unwrap().value(),
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
    Creator,
    Owner,
    FindUnit { slot: usize, faction: Faction },
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
            ExpressionEntity::Creator => context.creator,
            ExpressionEntity::Owner => context.owner,
            ExpressionEntity::FindUnit { slot, faction } => {
                WorldSystem::collect_factions(world, hashset! {*faction})
                    .into_iter()
                    .find_map(|(entity, unit)| match unit.slot == *slot {
                        true => Some(entity),
                        false => None,
                    })
                    .expect(&format!("No unit of {:?} found in {} slot", faction, slot))
            }
        }
    }
}
