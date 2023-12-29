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
    UnitSync,
}

impl GameState {
    pub fn change(self, world: &mut World) {
        debug!("Change state to {self}");
        world
            .get_resource_mut::<NextState<GameState>>()
            .unwrap()
            .set(self);
    }
    pub fn exit(&self, world: &mut World) {
        match self {
            GameState::MainMenu | GameState::Login => world.send_event(AppExit),
            GameState::CustomBattle | GameState::Battle => {
                GameTimer::get_mut(world).skip_to_end();
            }
            GameState::Shop | GameState::HeroEditor | GameState::HeroGallery => {
                Self::MainMenu.change(world)
            }
            GameState::TestsLoading
            | GameState::BattleTest
            | GameState::Restart
            | GameState::UnitSync
            | GameState::Loading => {}
        }
    }
}
