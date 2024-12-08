use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum Trigger {
    #[default]
    BattleStart,
    TurnEnd,
}
