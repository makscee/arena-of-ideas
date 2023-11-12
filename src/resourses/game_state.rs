use super::*;

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq, Debug, Hash, Default, States, Display)]
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
        debug!("Change state to {next}");
        world
            .get_resource_mut::<NextState<GameState>>()
            .unwrap()
            .set(next);
    }
}
