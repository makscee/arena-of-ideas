pub use super::*;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash, enum_utils::FromStr)]
pub enum VarName {
    DamageIncoming,
    DamageDealt,
    DamageTaken,
    DamageBlocked,
    HealthRestored,
    IncomingHeal,
    TargetCount,
    Value,
    SpawnHealth,
    StackCounter,
    PoisonTicksLeft,
    Charges,
    StolenDamage,
    StolenHealth,
    StealPercent,
    RemainingAbsorb,
    GlobalVar,
    HealLeft,
    OldHealth,
    OldAttack,
    DevourMultiplier,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum Expr {
    Const {
        value: i32,
    },
    Var {
        name: VarName,
    },
    Sum {
        a: Box<Expr>,
        b: Box<Expr>,
    },
    Sub {
        a: Box<Expr>,
        b: Box<Expr>,
    },
    Mul {
        a: Box<Expr>,
        b: Box<Expr>,
    },
    Inv {
        value: Box<Expr>,
    },
    FindStat {
        who: Who,
        stat: UnitStat,
    },
    WithVar {
        name: VarName,
        value: Box<Expr>,
        result: Box<Expr>,
    },
    UnitsInRange {
        max_distance: Option<Coord>,
        faction: Option<Faction>,
        status: Option<StatusName>,
        clan: Option<Clan>,
    },
    If {
        condition: Box<Condition>,
        then: Box<Expr>,
        r#else: Box<Expr>,
    },
}

impl Expr {
    pub fn calculate(&self, context: &EffectContext, model: &Model) -> i32 {
        match self {
            Self::Const { value } => *value,
            Self::Var { name } => {
                for unit in model.get_all(context) {
                    if unit.template.vars.contains_key(name) {
                        return unit.template.vars[name].calculate(context, model);
                    }
                }
                context.vars[name]
            }
            Self::Sum { a, b } => a.calculate(context, model) + b.calculate(context, model),
            Self::Sub { a, b } => a.calculate(context, model) - b.calculate(context, model),
            Self::Mul { a, b } => a.calculate(context, model) * b.calculate(context, model),
            Self::Inv { value } => 0 - value.calculate(context, model),
            Self::FindStat { who, stat } => {
                let target = model.get_who(*who, &context);
                target.stats.get(*stat)
            }
            Self::WithVar {
                name,
                value,
                result,
            } => {
                let mut context = context.clone();
                context
                    .vars
                    .insert(name.clone(), value.calculate(&context, model));
                result.calculate(&context, model)
            }
            Self::UnitsInRange {
                max_distance,
                faction,
                status,
                clan,
            } => {
                let owner = model.get_who(Who::Owner, &context);
                model
                    .units
                    .iter()
                    .filter(|unit| match faction {
                        Some(f) => unit.faction == *f,
                        None => true,
                    })
                    .filter(|unit| match max_distance {
                        Some(distance) => distance_between_units(owner, unit) < *distance,
                        None => true,
                    })
                    .filter(|unit| match status {
                        Some(status_name) => unit
                            .all_statuses
                            .iter()
                            .any(|unit_status| unit_status.status.name == *status_name),
                        None => true,
                    })
                    .count() as i32
            }
            Expr::If {
                condition,
                then,
                r#else,
            } => {
                if model.check_condition(condition, context) {
                    then.calculate(&context, model)
                } else {
                    r#else.calculate(&context, model)
                }
            }
        }
    }
}
