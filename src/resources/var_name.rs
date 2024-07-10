use super::*;

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
)]
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
    Lvl,
    Stacks,
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
    Direction,
    Alpha,
    T,
    Houses,
    Thickness,
    M1,
    M2,
    M3,
    TriggersDescription,
    TargetsDescription,
    EffectsDescription,
    UsedDefinitions,
}

impl ToCstr for VarName {
    fn cstr(self) -> Cstr {
        self.to_string().cstr()
    }
}
