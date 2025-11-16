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
    rarity,
    tier,
    visible,
    slot,
    side,
    unit_size,
    t,
    text,
    g,
    lives,
    price,
    unit,
    floor,
    stax,
    online,
    active,
    no_stats,
    scale,
}

impl VarName {
    pub fn is_stat(self) -> bool {
        match self {
            VarName::pwr | VarName::hp | VarName::dmg | VarName::stax => true,
            _ => false,
        }
    }
}
