use super::*;

#[derive(Clone, Copy, Deserialize, Serialize, Debug, Default, PartialEq, Eq, Hash, Display)]
pub enum VarName {
    #[default]
    None,
    Offset,
    Position,
    Name,
    Description,
    Hp,
    Pwr,
    Dmg,
    Value,
    Damage,
    Slot,
    Visible,
    Faction,
}
