use super::*;

#[derive(Clone, Copy, Deserialize, Serialize, Debug, Default, PartialEq, Eq, Hash, Display)]
pub enum VarName {
    #[default]
    None,
    Offset,
    Position,
    Name,
    Description,
    Index,
    Scale,
    Rotation,
    Hp,
    Pwr,
    Dmg,
    Value,
    Delta,
    Damage,
    Color,
    RarityColor,
    Slot,
    Visible,
    Faction,
    Charges,
    Polarity,
    Level,
}
