use super::*;

#[derive(
    Debug, Clone, Serialize, Deserialize, Default, AsRefStr, EnumIter, PartialEq, Eq, Hash,
)]
pub enum Trigger {
    #[default]
    BattleStart,
    TurnEnd,
    ChangeStat(VarName),
    BeforeDeath,
}
