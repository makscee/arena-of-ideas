use super::*;

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, AsRefStr, EnumIter, Display)]
pub enum Action {
    #[default]
    Noop,
    SetValue(Box<Expression>),
    AddValue(Box<Expression>),
    SubtractValue(Box<Expression>),
    SetTarget(Box<Expression>),
    MultipleTargets(Box<Expression>, Vec<Box<Action>>),
    DealDamage,
    Repeat(Box<Expression>, Vec<Box<Action>>),
}
