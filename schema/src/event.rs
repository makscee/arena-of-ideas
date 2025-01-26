use super::*;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Default, Clone, AsRefStr, EnumIter)]
pub enum Event {
    #[default]
    BattleStart,
    TurnEnd,
    UpdateStat(VarName),
}
