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
    Sub {
        a: Box<ExpressionInt>,
        b: Box<ExpressionInt>,
    },
    Mul {
        a: Box<ExpressionInt>,
        b: Box<ExpressionInt>,
    },
    Max {
        a: Box<ExpressionInt>,
        b: Box<ExpressionInt>,
    },
    If {
        condition: Box<Condition>,
        then: Box<ExpressionInt>,
        r#else: Box<ExpressionInt>,
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
        ability: AbilityName,
        var: VarName,
    },
    Negate {
        value: Box<ExpressionInt>,
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
            ExpressionInt::Sub { a, b } => {
                Ok(a.calculate(context, world, resources)?
                    - b.calculate(context, world, resources)?)
            }
            ExpressionInt::Mul { a, b } => {
                Ok(a.calculate(context, world, resources)?
                    * b.calculate(context, world, resources)?)
            }
            ExpressionInt::Max { a, b } => Ok(a
                .calculate(context, world, resources)?
                .max(b.calculate(context, world, resources)?)),
            ExpressionInt::Const { value } => Ok(*value),
            ExpressionInt::Var { var } => {
                context.vars.try_get_int(var).context("Failed to find var")
            }
            ExpressionInt::EntityVar { var, entity } => {
                ContextSystem::try_get_context(entity.calculate(context, world, resources)?, world)?
                    .vars
                    .try_get_int(var)
                    .context(format!("Var not found {}", var))
            }
            ExpressionInt::AbilityVar { ability, var } => {
                let faction = Faction::from_entity(context.owner, world);
                Ok(AbilityPool::get_var_int(resources, &faction, ability, var))
            }
            ExpressionInt::If {
                condition,
                then,
                r#else,
            } => match condition.calculate(context, world, resources)? {
                true => then.calculate(context, world, resources),
                false => r#else.calculate(context, world, resources),
            },
            ExpressionInt::Negate { value } => Ok(-value.calculate(context, world, resources)?),
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
        faction: ExpressionFaction,
    },
    RandomUnit {
        faction: ExpressionFaction,
    },
    SlotRelative {
        relation: Box<ExpressionInt>,
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
                let faction = faction.calculate(context, world, resources)?;
                SlotSystem::find_unit_by_slot(slot, &faction, world)
                    .context(format!("No unit of {:?} found in {} slot", faction, slot))
            }
            ExpressionEntity::RandomUnit { faction } => {
                let faction = faction.calculate(context, world, resources)?;
                <(&UnitComponent, &EntityComponent)>::query()
                    .iter(world)
                    .filter_map(|(unit, entity)| match unit.faction == faction {
                        true => Some(entity.entity),
                        false => None,
                    })
                    .choose(&mut thread_rng())
                    .context(format!("No units of {:?} found", faction))
            }
            ExpressionEntity::SlotRelative { relation } => {
                let unit = UnitSystem::get_unit(context.owner, world);
                let relation = relation.calculate(context, world, resources)?;
                let slot = (unit.slot as i32 + relation) as usize;
                let faction = unit.faction;
                SlotSystem::find_unit_by_slot(slot, &faction, world)
                    .context(format!("No unit of {:?} found in slot {}", faction, slot))
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
    Opposite { faction: Box<ExpressionFaction> },
    Var { var: VarName },
}

impl ExpressionFaction {
    pub fn calculate(
        &self,
        context: &Context,
        world: &legion::World,
        resources: &Resources,
    ) -> Result<Faction, Error> {
        match &self {
            ExpressionFaction::Owner => Ok(Faction::from_entity(context.owner, world)),
            ExpressionFaction::Target => Ok(Faction::from_entity(context.target, world)),
            ExpressionFaction::Parent => Ok(Faction::from_entity(context.parent.unwrap(), world)),

            ExpressionFaction::Opposite { faction } => {
                Ok(faction.calculate(context, world, resources)?.opposite())
            }
            ExpressionFaction::Var { var } => context
                .vars
                .try_get_faction(var)
                .context("Failed to get faction var"),
        }
    }
}
