mod nodes;
mod plugins;
pub mod prelude;
mod resources;
mod stdb;
mod ui;
mod utils;

use bevy::{app::PreStartup, diagnostic::FrameTimeDiagnosticsPlugin, state::app::AppExtStates};
use bevy_egui::{EguiContextSettings, EguiPlugin, EguiStartupSet};
use clap::{Parser, ValueEnum, command};
use include_dir::include_dir;
pub use prelude::*;

#[derive(Parser, Debug, Default, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(short, long)]
    mode: RunMode,
    #[arg(short, long)]
    extra: Option<String>,
}

static ARGS: OnceCell<Args> = OnceCell::new();
pub fn run_mode() -> &'static RunMode {
    &ARGS.get().unwrap().mode
}

#[derive(Debug, Clone, ValueEnum, Default, PartialEq, Eq)]
pub enum RunMode {
    #[default]
    Regular,
    Shop,
    Test,
    Sync,
    MigrationDownload,
    MigrationUpload,
}

fn main() {
    let mut app = App::new();
    let args = Args::try_parse().unwrap_or_default();
    ARGS.set(args.clone()).unwrap();
    unsafe {
        std::env::set_var("RUST_BACKTRACE", "1");
        std::env::set_var("RUST_LIB_BACKTRACE", "0");
    }
    let target = match args.mode {
        RunMode::Regular => GameState::Title,
        RunMode::Shop => GameState::Shop,
        RunMode::Test => GameState::TestScenariosRun,
        RunMode::Sync => GameState::ServerSync,
        RunMode::MigrationDownload => GameState::MigrationDownload,
        RunMode::MigrationUpload => GameState::MigrationUpload,
    };
    PersistentDataPlugin::load();
    parse_content_tree();
    GameState::set_target(target);
    let default_plugins = DefaultPlugins.set(LogPlugin {
        level: bevy::log::Level::DEBUG,
        filter: "info,debug,wgpu_core=warn,wgpu_hal=warn,naga=warn".into(),
        ..default()
    });
    app.add_systems(Startup, setup)
        .add_systems(OnEnter(GameState::Error), on_error_state)
        .add_plugins((default_plugins, FrameTimeDiagnosticsPlugin::new(10)))
        .add_loading_state(
            LoadingState::new(GameState::Loading)
                .continue_to_state(GameState::Loaded)
                .on_failure_continue_to_state(GameState::Error)
                .load_collection::<AudioAssets>()
                .with_dynamic_assets_file::<StandardDynamicAssetCollection>(
                    "ron/_dynamic.assets.ron",
                ),
        )
        .add_plugins(EguiPlugin {
            enable_multipass_for_primary_context: false,
        })
        .add_systems(
            PreStartup,
            configure_context.after(EguiStartupSet::InitContexts),
        )
        .add_plugins((
            UiPlugin,
            LoginPlugin,
            GameStatePlugin,
            NodeStatePlugin,
            RepresentationPlugin,
            GameTimerPlugin,
            WindowPlugin,
            MatchPlugin,
            PersistentDataPlugin,
            BattlePlugin,
            NodeExplorerPlugin,
        ))
        .add_plugins((
            OperationsPlugin,
            ConnectPlugin,
            ClientSettingsPlugin,
            TilePlugin,
            AudioPlugin,
            ConfirmationPlugin,
            AdminPlugin,
            StdbPlugin,
            NotificationsPlugin,
        ))
        .init_state::<GameState>();
    app.run();
}

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

fn setup(world: &mut World) {
    let ctx = world.query::<&EguiContext>().single(world).unwrap().get();
    egui_extras::install_image_loaders(&ctx);
    world.init_links();
}

fn on_error_state(world: &mut World) {
    app_exit(world)
}

fn configure_context(mut egui_settings: Query<&mut EguiContextSettings>) {
    egui_settings.single_mut().unwrap().run_manually = true;
}
