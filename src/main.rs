mod nodes;
mod plugins;
pub mod prelude;
mod resources;
mod stdb;
mod ui;
mod utils;

use bevy::{diagnostic::FrameTimeDiagnosticsPlugin, state::app::AppExtStates};
use clap::{command, Parser, ValueEnum};
use include_dir::include_dir;
use noisy_bevy::NoisyShaderPlugin;
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
        RunMode::Shop => GameState::Match,
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
        .add_plugins((default_plugins, FrameTimeDiagnosticsPlugin))
        .add_loading_state(
            LoadingState::new(GameState::Loading)
                .continue_to_state(GameState::Loaded)
                .on_failure_continue_to_state(GameState::Error)
                .load_collection::<AudioAssets>()
                .with_dynamic_assets_file::<StandardDynamicAssetCollection>(
                    "ron/_dynamic.assets.ron",
                ),
        )
        .add_plugins((
            bevy_egui::EguiPlugin,
            bevy_inspector_egui::quick::WorldInspectorPlugin::new().run_if(|| false),
            NoisyShaderPlugin,
        ))
        .add_plugins((
            UiPlugin,
            LoginPlugin,
            GameStatePlugin,
            TextColumnPlugin,
            CameraPlugin,
            NodeStatePlugin,
            RepresentationPlugin,
            GameTimerPlugin,
            WindowPlugin,
            BackgroundPlugin,
            StdbSyncPlugin,
            MatchPlugin,
            PersistentDataPlugin,
            BattlePlugin,
        ))
        .add_plugins((
            OperationsPlugin,
            ConnectPlugin,
            ClientSettingsPlugin,
            TilePlugin,
            AudioPlugin,
            ConfirmationPlugin,
            AdminPlugin,
            FusionEditorPlugin,
            StdbPlugin,
            IncubatorPlugin,
            NotificationsPlugin,
        ))
        .init_state::<GameState>();
    for n in NodeKind::iter() {
        n.register(&mut app);
    }
    app.run();
}

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

fn setup(world: &mut World) {
    if let Some(ctx) = egui_context(world) {
        egui_extras::install_image_loaders(&ctx);
    }
    CameraPlugin::respawn_camera(world);
}

fn on_error_state(world: &mut World) {
    app_exit(world)
}
