use super::*;
use bevy::ecs::schedule::NextState;
use spacetimedb_lib::de::serde::DeserializeWrapper;

#[derive(AssetCollection, Resource)]
pub struct GameAssetsHandles {
    #[asset(key = "global_settings")]
    global_settings: Handle<GlobalSettingsAsset>,
    #[asset(key = "custom_battle")]
    custom_battle: Handle<BattleData>,
}

#[derive(Resource)]
pub struct GameAssets {
    pub global_settings: GlobalSettings,
    pub custom_battle: BattleData,
}

#[derive(Deserialize, Asset, TypePath)]
pub struct GlobalSettingsAsset {
    settings: DeserializeWrapper<GlobalSettings>,
}

impl GameAssets {
    pub fn get(world: &World) -> &Self {
        world.resource::<Self>()
    }
}

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Loaded), Self::setup);
    }
}

impl LoadingPlugin {
    fn setup(world: &mut World) {
        let global_settings = world
            .resource::<Assets<GlobalSettingsAsset>>()
            .get(&world.resource::<GameAssetsHandles>().global_settings)
            .unwrap()
            .settings
            .0
            .clone();
        let custom_battle = world
            .resource::<Assets<BattleData>>()
            .get(&world.resource::<GameAssetsHandles>().custom_battle)
            .unwrap()
            .clone();

        world.insert_resource(GameAssets {
            global_settings,
            custom_battle,
        });

        world
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Connect);
    }
}
