mod plugins;
pub mod prelude;
mod resources;
mod stdb;
mod utils;

use bevy::{
    diagnostic::FrameTimeDiagnosticsPlugin, log::LogPlugin, render::camera::ClearColor,
    state::app::AppExtStates,
};
use clap::{command, Parser, ValueEnum};
use include_dir::{include_dir, Dir};
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

pub static ARGS: OnceCell<Args> = OnceCell::new();

#[derive(Debug, Clone, ValueEnum, Default)]
pub enum RunMode {
    #[default]
    Regular,
    Custom,
    Shop,
    Editor,
    Test,
    Sync,
    MigrationDownload,
    MigrationUpload,
    Query,
    Admin,
    Incubator,
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
        RunMode::Regular => GameState::Admin,
        RunMode::Custom => GameState::CustomBattle,
        RunMode::Shop => GameState::Shop,
        RunMode::Editor => GameState::Editor,
        RunMode::Test => GameState::TestScenariosRun,
        RunMode::Sync => GameState::ServerSync,
        RunMode::MigrationDownload => GameState::MigrationDownload,
        RunMode::MigrationUpload => GameState::MigrationUpload,
        RunMode::Query => GameState::Query,
        RunMode::Admin => GameState::Admin,
        RunMode::Incubator => GameState::Incubator,
    };
    load_client_settings();
    load_client_state();
    init_style_map();
    parse_content_tree();
    GameState::set_target(target);
    let default_plugins = DefaultPlugins.set(LogPlugin {
        level: bevy::log::Level::DEBUG,
        filter: "info,debug,wgpu_core=warn,wgpu_hal=warn,naga=warn".into(),
        ..default()
    });
    let b = Battle {
        left: [Unit {
            name: "Left 1".into(),
            stats: Some(UnitStats {
                pwr: 1,
                hp: 3,
                ..default()
            }),
            ..default()
        }]
        .into(),
        right: [Unit {
            name: "Right 1".into(),
            stats: Some(UnitStats {
                pwr: 1,
                hp: 4,
                ..default()
            }),
            ..default()
        }]
        .into(),
    };
    let a = b.run();
    dbg!(&a);
    for a in a {
        a.cstr().print();
    }
    return;
    app.insert_resource(ClearColor(EMPTINESS.to_color()))
        .add_systems(Startup, setup)
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
            bevy_inspector_egui::quick::WorldInspectorPlugin::new(),
            NoisyShaderPlugin,
        ))
        .add_plugins((
            UiPlugin,
            LoginPlugin,
            GameStatePlugin,
            WidgetsPlugin,
            TextColumnPlugin,
            CameraPlugin,
            NodeStatePlugin,
            RepresentationPlugin,
            GameTimerPlugin,
            WindowPlugin,
            BackgroundPlugin,
            HeroPlugin,
        ))
        .add_plugins((
            OperationsPlugin,
            ProfilePlugin,
            ConnectPlugin,
            ClientSettingsPlugin,
            TilePlugin,
            AudioPlugin,
            ConfirmationPlugin,
            AdminPlugin,
        ))
        .init_state::<GameState>()
        .init_resource::<NotificationsResource>();
    for n in NodeKind::iter() {
        n.register(&mut app);
    }
    if !cfg!(debug_assertions) {
        app.add_plugins(bevy_panic_handler::PanicHandler::new().build());
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
