use super::*;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Serialize, Deserialize, AsRefStr, EnumIter, PartialEq, Default)]
#[serde(deny_unknown_fields)]
pub enum Action {
    #[default]
    noop,
    debug(Box<Expression>),
    set_value(Box<Expression>),
    add_value(Box<Expression>),
    subtract_value(Box<Expression>),
    add_target(Box<Expression>),
    deal_damage,
    heal_damage,
    use_ability,
    apply_status,
    repeat(Box<Expression>, Vec<Box<Action>>),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Actions(pub Vec<Box<Action>>);

impl From<Vec<Action>> for Actions {
    fn from(value: Vec<Action>) -> Self {
        Self(value.into_iter().map(|v| Box::new(v)).collect())
    }
}

impl std::ops::Index<usize> for Actions {
    type Output = Action;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}
