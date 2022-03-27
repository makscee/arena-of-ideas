pub use super::*;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub enum VarName {
    DamageDealt,
    DamageBlocked,
    HealthRestored,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum Expr {
    Const { value: R32 },
    Var { name: VarName },
    Sum { a: Box<Expr>, b: Box<Expr> },
    Mul { a: Box<Expr>, b: Box<Expr> },
    FindStat { who: Who, stat: UnitStat },
}

impl Expr {
    pub fn calculate(&self, context: &EffectContext, logic: &Logic) -> R32 {
        match self {
            Self::Const { value } => *value,
            Self::Var { name } => context.vars[name],
            Self::Sum { a, b } => a.calculate(context, logic) + b.calculate(context, logic),
            Self::Mul { a, b } => a.calculate(context, logic) * b.calculate(context, logic),
            Self::FindStat { who, stat } => {
                let target = context.get(*who).unwrap();
                let target = logic
                    .model
                    .units
                    .get(&target)
                    .or_else(|| logic.model.dead_units.get(&target))
                    .unwrap();
                target.stat(*stat)
            }
        }
    }
}
