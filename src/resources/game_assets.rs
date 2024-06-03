use super::*;
use spacetimedb_lib::de::serde::DeserializeWrapper;

#[derive(AssetCollection, Resource)]
pub struct GameAssetsHandles {
    #[asset(key = "global_settings")]
    global_settings_handle: Handle<GlobalSettingsAsset>,
}

#[derive(Resource)]
pub struct GameAssets {
    pub global_settings: GlobalSettings,
}

#[derive(Deserialize, Asset, TypePath)]
pub struct GlobalSettingsAsset {
    settings: DeserializeWrapper<GlobalSettings>,
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
            .get(&world.resource::<GameAssetsHandles>().global_settings_handle)
            .unwrap()
            .settings
            .0
            .clone();

        world.insert_resource(GameAssets { global_settings });
    }
}
