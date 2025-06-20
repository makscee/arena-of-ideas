use super::*;

#[derive(
    Debug,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    Default,
    Clone,
    Copy,
    AsRefStr,
    EnumIter,
    Display,
    Hash,
)]
pub enum Event {
    #[default]
    BattleStart,
    TurnEnd,
    UpdateStat(VarName),
    Death(u64),
    OutgoingDamage(u64, u64),
    IncomingDamage(u64, u64),
}
