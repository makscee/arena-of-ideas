mod nodes;
mod plugins;
pub mod prelude;
mod resources;
mod stdb;
#[cfg(test)]
mod tests;
mod ui;
mod utils;

use bevy::{
    app::PreStartup, asset::AssetPlugin, diagnostic::FrameTimeDiagnosticsPlugin,
    state::app::AppExtStates,
};
use bevy_egui::{EguiContextSettings, EguiPlugin, EguiStartupSet};
use clap::{Parser, ValueEnum};
use include_dir::include_dir;
pub use prelude::*;

use crate::plugins::stdb_auth::StdbAuthPlugin;

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
    WorldDownload,
    WorldUpload,
}

fn fmt_layer(_app: &mut App) -> Option<bevy::log::BoxedFmtLayer> {
    Some(Box::new(
        bevy::log::tracing_subscriber::fmt::Layer::default().with_ansi(true),
    ))
}

pub fn run() {
    let mut app = App::new();
    let args = Args::try_parse().unwrap_or_default();
    ARGS.set(args.clone()).unwrap();
    unsafe {
        std::env::set_var("RUST_BACKTRACE", "1");
        std::env::set_var("RUST_LIB_BACKTRACE", "0");
        std::env::set_var("NO_COLOR", "1");
    }
    let target = match args.mode {
        RunMode::Regular => GameState::Title,
        RunMode::Shop => GameState::Shop,
        RunMode::Test => GameState::TestScenariosRun,
        RunMode::Sync => GameState::ServerSync,
        RunMode::WorldDownload => GameState::WorldDownload,
        RunMode::WorldUpload => GameState::WorldUpload,
    };
    PersistentDataPlugin::load();
    GAME_TIMER.set(default()).unwrap();
    parse_content_tree();
    init_completer();
    GameState::set_target(target);
    let default_plugins = DefaultPlugins
        .set(bevy::log::LogPlugin {
            level: pd().client_settings.log_level.into(),
            // filter: "info,debug,wgpu_core=warn,wgpu_hal=warn,naga=warn".into(),
            fmt_layer: fmt_layer,
            ..default()
        })
        .set(AssetPlugin {
            file_path: "assets".to_string(),
            ..default()
        });
    app.add_systems(Startup, setup)
        .add_systems(OnEnter(GameState::Error), on_error_state)
        .add_plugins((default_plugins, FrameTimeDiagnosticsPlugin::new(10)))
        .add_systems(
            PreStartup,
            (setup_camera_system.before(EguiStartupSet::InitContexts),),
        )
        .add_plugins(EguiPlugin::default())
        .add_systems(
            PreStartup,
            configure_context.after(EguiStartupSet::InitContexts),
        )
        .add_plugins((
            UiPlugin,
            LoginPlugin,
            GameStatePlugin,
            GameTimerPlugin,
            WindowPlugin,
            MatchPlugin,
            PersistentDataPlugin,
            BattlePlugin,
            BattleEditorPlugin,
            IncubatorPlugin,
        ))
        .add_plugins((
            OperationsPlugin,
            ConnectPlugin,
            ClientSettingsPlugin,
            TilePlugin,
            AudioPlugin,
            ConfirmationPlugin,
            AdminPlugin,
            WorldMigrationPlugin,
            StdbPlugin,
            StdbAuthPlugin,
            NotificationsPlugin,
        ))
        .init_state::<GameState>();
    app.run();
}

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

fn setup(world: &mut World) {
    let ctx = world.query::<&EguiContext>().single(world).unwrap().get();
    egui_extras::install_image_loaders(&ctx);
    pd_save_settings();
}

fn on_error_state(world: &mut World) {
    app_exit(world)
}

fn configure_context(mut egui_settings: Query<&mut EguiContextSettings>) {
    egui_settings.single_mut().unwrap().run_manually = true;
}

fn setup_camera_system(mut commands: Commands) {
    commands.spawn(bevy::camera::Camera2d);
}
