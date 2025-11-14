use super::*;

#[derive(
    Debug,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    Default,
    AsRefStr,
    EnumIter,
    PartialEq,
    Eq,
    Hash,
    Display,
)]
pub enum Trigger {
    #[default]
    BattleStart,
    TurnEnd,
    BeforeDeath,
    AllyDeath,
    ChangeStat(VarName),
    ChangeOutgoingDamage,
    ChangeIncomingDamage,
}
