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
    HeroEditor,
    HeroGallery,
    Login,
    Profile,
}

impl GameState {
    pub fn change(self, world: &mut World) {
        debug!("Change state to {self}");
        world
            .get_resource_mut::<NextState<GameState>>()
            .unwrap()
            .set(self);
    }
}
