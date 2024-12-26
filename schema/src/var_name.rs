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
)]
pub enum VarName {
    #[default]
    none,
    position,
    offset,
    hp,
    pwr,
    data,
    name,
    description,
    color,
    lvl,
    index,
    visible,
    slot,
    side,
    unit_size,
    t,
}
