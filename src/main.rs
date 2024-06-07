mod components;
mod module_bindings;
mod plugins;
pub mod prelude;
mod resources;
mod utils;

pub use prelude::*;

fn main() {
    let mut app = App::new();
    app.init_state::<GameState>()
        .add_plugins(DefaultPlugins)
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
        .add_plugins((LoadingPlugin, LoginPlugin, BattlePlugin, TeamPlugin))
        .run();
}
