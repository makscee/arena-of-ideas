mod components;
mod plugins;
pub mod prelude;
mod resources;
mod stdb;
mod utils;

use bevy::{
    diagnostic::FrameTimeDiagnosticsPlugin, log::LogPlugin, render::camera::ClearColor,
    sprite::Material2dPlugin,
};
use clap::{command, Parser, ValueEnum};
use noisy_bevy::NoisyShaderPlugin;
pub use prelude::*;

#[derive(Parser, Debug, Default, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(short, long)]
    mode: RunMode,
    #[arg(short, long)]
    path: Option<String>,
}

pub static ARGS: OnceCell<Args> = OnceCell::new();

#[derive(Debug, Clone, ValueEnum, Default)]
pub enum RunMode {
    #[default]
    Regular,
    Custom,
    Shop,
    Test,
    Sync,
    ArchiveDownload,
    ArchiveUpload,
}

pub type GID = u64;

fn main() {
    let mut app = App::new();
    let args = Args::try_parse().unwrap_or_default();
    ARGS.set(args.clone()).unwrap();
    std::env::set_var("RUST_BACKTRACE", "1");
    std::env::set_var("RUST_LIB_BACKTRACE", "0");
    let target = match args.mode {
        RunMode::Regular => GameState::Title,
        RunMode::Custom => GameState::CustomBattle,
        RunMode::Shop => GameState::Shop,
        RunMode::Test => GameState::TestScenariosRun,
        RunMode::Sync => GameState::ServerSync,
        RunMode::ArchiveDownload => GameState::GameArchiveDownload,
        RunMode::ArchiveUpload => GameState::GameArchiveUpload,
    };
    load_client_settings();
    GameState::set_target(target);
    let default_plugins = DefaultPlugins.set(LogPlugin {
        level: bevy::log::Level::DEBUG,
        filter: "info,debug,wgpu_core=warn,wgpu_hal=warn,naga=warn".into(),
        ..default()
    });
    app.init_state::<GameState>()
        .insert_resource(ClearColor(EMPTINESS.to_color()))
        .add_systems(Startup, setup)
        .add_systems(Update, update)
        .add_systems(OnEnter(GameState::Error), on_error_state)
        .add_plugins((default_plugins, FrameTimeDiagnosticsPlugin))
        .add_loading_state(
            LoadingState::new(GameState::Loading)
                .continue_to_state(GameState::Loaded)
                .on_failure_continue_to_state(GameState::Error)
                .load_collection::<GameAssetsHandles>()
                .with_dynamic_assets_file::<StandardDynamicAssetCollection>(
                    "ron/_dynamic.assets.ron",
                ),
        )
        .add_loading_state(
            LoadingState::new(GameState::TestScenariosLoad)
                .continue_to_state(GameState::TestScenariosRun)
                .load_collection::<TestScenarios>(),
        )
        .add_plugins(RonAssetPlugin::<GlobalSettingsAsset>::new(&[
            "global_settings.ron",
        ]))
        .add_plugins(Material2dPlugin::<ShapeMaterial>::default())
        .add_plugins(Material2dPlugin::<CurveMaterial>::default())
        .add_plugins(RonAssetPlugin::<BattleData>::new(&["battle.ron"]))
        .add_plugins(RonAssetPlugin::<PackedUnit>::new(&["unit.ron"]))
        .add_plugins(RonAssetPlugin::<House>::new(&["house.ron"]))
        .add_plugins(RonAssetPlugin::<TestScenario>::new(&["scenario.ron"]))
        .add_plugins(RonAssetPlugin::<Representation>::new(&["rep.ron"]))
        .add_plugins(RonAssetPlugin::<Animations>::new(&["anim.ron"]))
        .add_plugins(RonAssetPlugin::<Vfx>::new(&["vfx.ron"]))
        .add_plugins(bevy_egui::EguiPlugin)
        .add_plugins(NoisyShaderPlugin)
        .add_plugins((
            LoadingPlugin,
            UiPlugin,
            LoginPlugin,
            ActionPlugin,
            BattlePlugin,
            TeamPlugin,
            GameStatePlugin,
            TestScenariosPlugin,
            ServerSyncPlugin,
            WidgetsPlugin,
            RepresentationPlugin,
            CameraPlugin,
            TextColumnPlugin,
            ShopPlugin,
            UnitPlugin,
        ))
        .add_plugins((
            OperationsPlugin,
            ProfilePlugin,
            StdbQueryPlugin,
            ConnectPlugin,
            TableViewPlugin,
            GameArchivePlugin,
            ClientSettingsPlugin,
        ))
        .run();
}

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

fn setup(world: &mut World) {
    if let Some(ctx) = egui_context(world) {
        egui_extras::install_image_loaders(&ctx);
    }
}

fn update(time: Res<Time>) {
    gt().update(time.delta_seconds())
}

fn on_error_state(world: &mut World) {
    app_exit(world)
}
