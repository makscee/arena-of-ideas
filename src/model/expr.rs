pub use super::*;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
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
    pub fn calculate(&self, context: &EffectContext, logic: &Logic) -> i32 {
        match self {
            Self::Const { value } => *value,
            Self::Var { name } => context.vars[name],
            Self::Sum { a, b } => a.calculate(context, logic) + b.calculate(context, logic),
            Self::Sub { a, b } => a.calculate(context, logic) - b.calculate(context, logic),
            Self::Mul { a, b } => a.calculate(context, logic) * b.calculate(context, logic),
            Self::FindStat { who, stat } => {
                let target = context.get(*who).unwrap();
                let target = logic
                    .model
                    .units
                    .get(&target)
                    .or_else(|| logic.model.dead_units.get(&target))
                    .unwrap();
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
                    .insert(name.clone(), value.calculate(&context, logic));
                result.calculate(&context, logic)
            }
            Self::UnitsInRange {
                max_distance,
                faction,
                status,
                clan,
            } => {
                let from = context
                    .from
                    .and_then(|id| logic.model.units.get(&id))
                    .expect("From not found");
                logic
                    .model
                    .units
                    .iter()
                    .filter(|unit| match faction {
                        Some(f) => unit.faction == *f,
                        None => true,
                    })
                    .filter(|unit| match max_distance {
                        Some(distance) => distance_between_units(from, unit) < *distance,
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
                if logic.check_condition(condition, context) {
                    then.calculate(&context, logic)
                } else {
                    r#else.calculate(&context, logic)
                }
            }
        }
    }
}
