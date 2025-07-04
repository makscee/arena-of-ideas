use super::*;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Serialize, Deserialize, AsRefStr, EnumIter, PartialEq, Default, Hash)]
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
    repeat(Box<Expression>, Vec<Box<Action>>),
}
