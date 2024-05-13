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
    LastBattle,
    ClipboardBattle,
    Battle,
    Shop,
    HeroEditor,
    HeroTable,
    HeroGallery,
    Login,
    AssetSync,
    MigrationSave,
    MigrationUpload,
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
            GameState::MainMenu | GameState::Login => {
                AlertPlugin::add(
                    Some("Exit?".to_owned()),
                    "Are you sure you want to exit the the game?".to_owned(),
                    Some(Box::new(|world: &mut World| {
                        world.send_event(AppExit);
                    })),
                );
            }
            GameState::CustomBattle
            | GameState::ClipboardBattle
            | GameState::LastBattle
            | GameState::Battle => {
                GameTimer::get().skip_to_end();
            }
            GameState::Shop | GameState::HeroEditor | GameState::HeroGallery => {
                Self::MainMenu.change(world)
            }
            GameState::HeroTable => Self::HeroEditor.change(world),
            GameState::TestsLoading
            | GameState::BattleTest
            | GameState::Restart
            | GameState::AssetSync
            | GameState::MigrationSave
            | GameState::MigrationUpload
            | GameState::Loading => {}
        }
    }
}
