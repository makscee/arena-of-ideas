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
    strum_macros::VariantNames,
)]
#[repr(u8)]
pub enum VarName {
    #[default]
    index,
    max_index,
    value,
    position,
    extra_position,
    offset,
    pwr,
    hp,
    dmg,
    data,
    player_name,
    unit_name,
    house_name,
    ability_name,
    status_name,
    description,
    color,
    lvl,
    xp,
    rarity,
    tier,
    visible,
    slot,
    side,
    unit_size,
    t,
    text,
    charges,
    g,
    lives,
    price,
    unit,
    floor,
    round,
    action_limit,
    stacks,
    online,
    active,
    actions_limit,
}

impl VarName {
    pub fn is_stat(self) -> bool {
        match self {
            VarName::pwr | VarName::hp | VarName::dmg => true,
            _ => false,
        }
    }
}
