mod components;
mod login_menu_system;
mod materials;
mod module_bindings;
mod plugins;
mod prelude;
pub mod resourses;
mod utils;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use prelude::*;
pub use spacetimedb_sdk;

use clap::{Parser, ValueEnum};
// use spacetimedb_sdk::{
//     identity::{once_on_connect, Credentials},
//     Address,
// };

#[derive(Parser, Debug, Default)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    mode: RunMode,
}

#[derive(Debug, Clone, ValueEnum, Default)]
enum RunMode {
    #[default]
    Regular,
    Clean,
    Test,
}

// const SPACETIMEDB_URI: &str = "http://localhost:3001";
// const DB_NAME: &str = "aoi";

// fn on_connected(creds: &Credentials, _client_address: Address) {
//     println!("{creds:?} {_client_address:?}");
// }
// /// Register all the callbacks our app will use to respond to database events.
// fn register_callbacks() {
//     // When we receive our `Credentials`, save them to a file.
//     once_on_connect(on_connected);
// }

fn main() {
    // register_callbacks();
    // connect(SPACETIMEDB_URI, DB_NAME, None).expect("Failed to connect");
    // loop {}
    // return;
    let args = Args::try_parse().unwrap_or_default();
    let next_state = match args.mode {
        RunMode::Regular => GameState::MainMenu,
        RunMode::Clean => GameState::MainMenuClean,
        RunMode::Test => GameState::TestsLoading,
    };
    App::new()
        .add_state::<GameState>()
        .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.1)))
        .insert_resource(PkvStore::new("makscee", "arena_of_ideas"))
        .add_plugins((
            DefaultPlugins
                .set(AssetPlugin {
                    watch_for_changes: ChangeWatcher::with_delay(Duration::from_millis(100)),
                    ..default()
                })
                .set(LogPlugin {
                    level: bevy::log::Level::DEBUG,
                    filter: "info,debug,wgpu_core=warn,wgpu_hal=warn,naga=warn".into(),
                })
                .set(bevy::window::WindowPlugin {
                    primary_window: Some(bevy::prelude::Window {
                        title: "Arena of Ideas".into(),
                        ..default()
                    }),
                    ..default()
                }),
            FrameTimeDiagnosticsPlugin,
        ))
        .add_loading_state(LoadingState::new(GameState::Loading).continue_to_state(next_state))
        .add_loading_state(
            LoadingState::new(GameState::TestsLoading).continue_to_state(GameState::BattleTest),
        )
        .add_dynamic_collection_to_loading_state::<_, StandardDynamicAssetCollection>(
            GameState::Loading,
            "ron/dynamic.assets.ron",
        )
        .add_collection_to_loading_state::<_, Options>(GameState::Loading)
        .add_collection_to_loading_state::<_, Pools>(GameState::Loading)
        .add_collection_to_loading_state::<_, TestScenarios>(GameState::TestsLoading)
        .add_systems(PreUpdate, update)
        .add_systems(PostUpdate, detect_changes)
        .add_plugins(DefaultPickingPlugins)
        .add_plugins(bevy_kira_audio::AudioPlugin)
        .add_plugins(Material2dPlugin::<ShapeMaterial>::default())
        .add_plugins(Material2dPlugin::<CurveMaterial>::default())
        .add_plugins(RonAssetPlugin::<PackedUnit>::new(&["unit.ron"]))
        .add_plugins(RonAssetPlugin::<House>::new(&["house.ron"]))
        .add_plugins(RonAssetPlugin::<CustomBattleData>::new(&["battle.ron"]))
        .add_plugins(RonAssetPlugin::<Representation>::new(&["rep.ron"]))
        .add_plugins(RonAssetPlugin::<Animations>::new(&["anim.ron"]))
        .add_plugins(RonAssetPlugin::<TestScenario>::new(&["scenario.ron"]))
        .add_plugins(RonAssetPlugin::<Vfx>::new(&["vfx.ron"]))
        .add_plugins(RonAssetPlugin::<Ladder>::new(&["ladder.ron"]))
        .add_plugins((
            MainMenuPlugin,
            RestartPlugin,
            CustomBattlePlugin,
            PoolsPlugin,
            ActionPlugin,
            UnitPlugin,
            RepresentationPlugin,
            ShopPlugin,
            BattlePlugin,
            TestPlugin,
            SettingsPlugin,
            AudioPlugin,
            HeroEditorPlugin,
            HeroGallery,
            CameraPlugin,
        ))
        .add_systems(Update, input_world)
        .init_resource::<UserName>()
        .init_resource::<Password>()
        .init_resource::<GameTimer>()
        .register_type::<VarState>()
        .register_type::<VarStateDelta>()
        .add_systems(Update, show_build_version)
        .run();
}

fn update(mut timer: ResMut<GameTimer>, time: Res<Time>, audio: Res<AudioData>) {
    if let Some(play_delta) = audio.play_delta {
        timer.advance_play(play_delta);
    } else {
        timer.advance_play(time.delta_seconds());
    }
}

fn input_world(world: &mut World) {
    let input = world.get_resource::<Input<KeyCode>>().unwrap();
    if !input.pressed(KeyCode::ControlLeft) {
        return;
    }
    if input.just_pressed(KeyCode::R) {
        if input.pressed(KeyCode::ShiftLeft) {
            let mut pd = PersistentData::load(world);
            pd.last_state = None;
            pd.save(world).unwrap();
        }
        GameState::change(GameState::Restart, world);
    }
}

fn detect_changes(
    mut unit_events: EventReader<AssetEvent<PackedUnit>>,
    mut rep_events: EventReader<AssetEvent<Representation>>,
    mut battle_state_events: EventReader<AssetEvent<CustomBattleData>>,
    mut vfx_events: EventReader<AssetEvent<Vfx>>,
    mut state: ResMut<NextState<GameState>>,
) {
    if unit_events.into_iter().any(|x| match x {
        AssetEvent::Modified { .. } => true,
        _ => false,
    }) || rep_events.into_iter().any(|x| match x {
        AssetEvent::Modified { .. } => true,
        _ => false,
    }) || battle_state_events.into_iter().any(|x| match x {
        AssetEvent::Modified { .. } => true,
        _ => false,
    }) || vfx_events.into_iter().any(|x| match x {
        AssetEvent::Modified { .. } => true,
        _ => false,
    }) {
        state.set(GameState::Loading)
    }
}

const VERSION: &str = env!("CARGO_PKG_VERSION");
fn show_build_version(world: &mut World) {
    let ctx = &egui_context(world);
    Area::new("build version")
        .anchor(Align2::LEFT_BOTTOM, [10.0, -10.0])
        .show(ctx, |ui| {
            if let Some(fps) = world
                .resource::<DiagnosticsStore>()
                .get(FrameTimeDiagnosticsPlugin::FPS)
            {
                if let Some(fps) = fps.smoothed() {
                    ui.label(format!("fps: {fps:.0}"));
                }
            }
            ui.label(format!("arena-of-ideas {VERSION}"));
        });
}
