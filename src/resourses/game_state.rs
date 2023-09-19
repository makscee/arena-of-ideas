use super::*;

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
pub enum GameState {
    #[default]
    AssetLoading,
    ScenariosLoading,
    Next,
    Restart,
    Battle,
    Shop,
    BattleTest,
}
