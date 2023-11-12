use super::*;

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
pub enum GameState {
    #[default]
    Loading,
    MainMenu,
    TestsLoading,
    BattleTest,
    Restart,
    CustomBattle,
    Battle,
    Shop,
}

impl GameState {
    pub fn change(next: GameState, world: &mut World) {
        world
            .get_resource_mut::<NextState<GameState>>()
            .unwrap()
            .set(next);
    }
}
