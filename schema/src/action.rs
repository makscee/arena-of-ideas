use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, AsRefStr, EnumIter, PartialEq, Default)]
#[serde(deny_unknown_fields)]
pub enum Action {
    #[default]
    Noop,
    Debug(Box<Expression>),
    SetValue(Box<Expression>),
    AddValue(Box<Expression>),
    SubtractValue(Box<Expression>),
    SetTarget(Box<Expression>),
    DealDamage,
    UseAbility,
    MultipleTargets(Box<Expression>, Vec<Box<Action>>),
    Repeat(Box<Expression>, Vec<Box<Action>>),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Actions(pub Vec<Box<Action>>);

impl From<Vec<Action>> for Actions {
    fn from(value: Vec<Action>) -> Self {
        Self(value.into_iter().map(|v| Box::new(v)).collect())
    }
}
