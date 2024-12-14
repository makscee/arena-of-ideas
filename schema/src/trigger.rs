use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, AsRefStr, EnumIter, PartialEq, Eq)]
pub enum Trigger {
    #[default]
    BattleStart,
    TurnEnd,
}
