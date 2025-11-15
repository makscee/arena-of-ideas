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
    BeforeStrike(u64, u64),
    AfterStrike(u64, u64),
    StatusApplied(u64, u64, u64),
    UpdateStat(VarName),
    Death(u64),
    OutgoingDamage(u64, u64),
    IncomingDamage(u64, u64),
    DamageDealt(u64, u64, i32),
}
