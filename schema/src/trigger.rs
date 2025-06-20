use super::*;

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, Default, AsRefStr, EnumIter, PartialEq, Eq, Hash,
)]
pub enum Trigger {
    #[default]
    BattleStart,
    TurnEnd,
    BeforeDeath,
    ChangeStat(VarName),
    ChangeOutgoingDamage,
    ChangeIncomingDamage,
}
