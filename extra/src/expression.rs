use std::{hash::Hasher, mem};

use super::*;

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, EnumIter, AsRefStr)]
pub enum Expression {
    #[default]
    One,
    Zero,
    Var(VarName),
    Value(VarValue),

    S(String),
}

impl Expression {
    pub fn get_value(&self, entity: Entity, world: &World) -> Option<VarValue> {
        match self {
            Expression::One => Some(1.into()),
            Expression::Zero => Some(0.into()),
            Expression::Var(var) => NodeState::get_var_e(*var, entity, world),
            Expression::Value(v) => Some(v.clone()),
            Expression::S(s) => Some(s.clone().into()),
        }
    }
}

impl std::hash::Hash for Expression {
    fn hash<H: Hasher>(&self, state: &mut H) {
        mem::discriminant(self).hash(state);
        match self {
            Expression::One | Expression::Zero => {}
            Expression::Var(v) => v.hash(state),
            Expression::Value(v) => v.hash(state),
            Expression::S(v) => v.hash(state),
        }
    }
}
