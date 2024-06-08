mod components;
mod module_bindings;
mod plugins;
pub mod prelude;
mod resources;
mod utils;

use bevy::log::LogPlugin;
pub use prelude::*;

fn main() {
    let mut app = App::new();
    GameState::set_target(GameState::CustomBattle);
    let default_plugins = DefaultPlugins.set(LogPlugin {
        level: bevy::log::Level::DEBUG,
        filter: "info,debug,wgpu_core=warn,wgpu_hal=warn,naga=warn".into(),
        ..default()
    });
    app.init_state::<GameState>()
        .add_plugins(default_plugins)
        .add_loading_state(
            LoadingState::new(GameState::Loading)
                .continue_to_state(GameState::Loaded)
                .load_collection::<GameAssetsHandles>()
                .with_dynamic_assets_file::<StandardDynamicAssetCollection>(
                    "ron/_dynamic.assets.ron",
                ),
        )
        .add_plugins(RonAssetPlugin::<GlobalSettingsAsset>::new(&[
            "global_settings.ron",
        ]))
        .add_plugins(RonAssetPlugin::<BattleData>::new(&["battle.ron"]))
        .add_plugins((
            LoadingPlugin,
            LoginPlugin,
            ActionPlugin,
            BattlePlugin,
            TeamPlugin,
            GameStateGraphPlugin,
        ))
        .run();
}
