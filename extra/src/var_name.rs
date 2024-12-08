use super::*;

#[allow(non_camel_case_types)]
#[derive(
    Clone,
    Copy,
    Deserialize,
    Serialize,
    Debug,
    Default,
    PartialEq,
    Eq,
    Hash,
    Display,
    AsRefStr,
    EnumString,
    EnumIter,
    Reflect,
)]
pub enum VarName {
    #[default]
    none,
    position,
    hp,
    pwr,
    data,
    name,
    description,
    color,
    lvl,
    index,
}
