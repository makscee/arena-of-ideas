pub use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum Expr {
    Const { value: R32 },
    Var { name: String },
    Sum { a: Box<Expr>, b: Box<Expr> },
    Mul { a: Box<Expr>, b: Box<Expr> },
    FindMaxHealth { who: Who },
}

impl Expr {
    pub fn calculate(&self, context: &EffectContext, logic: &Logic) -> R32 {
        match self {
            Self::Const { value } => *value,
            Self::Var { name } => {
                todo!()
            }
            Self::Sum { a, b } => a.calculate(context, logic) + b.calculate(context, logic),
            Self::Mul { a, b } => a.calculate(context, logic) * b.calculate(context, logic),
            Self::FindMaxHealth { who } => {
                let target = context.get(*who).unwrap();
                let target = logic
                    .model
                    .units
                    .get(&target)
                    .or_else(|| logic.model.dead_units.get(&target))
                    .unwrap();
                target.max_hp
            }
        }
    }
}
