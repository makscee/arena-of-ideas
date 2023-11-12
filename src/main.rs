mod components;
mod login_menu_system;
mod materials;
mod module_bindings;
mod plugins;
mod prelude;
pub mod resourses;
mod utils;
use prelude::*;
pub use spacetimedb_sdk;

use clap::{Parser, ValueEnum};
use spacetimedb_sdk::{
    identity::{once_on_connect, Credentials},
    Address,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Mode to run in: Test, Shop or CustomBattle
    #[arg(short, long)]
    mode: RunMode,
}

#[derive(Debug, Clone, ValueEnum)]
enum RunMode {
    CustomBattle,
    Shop,
    Test,
}

const SPACETIMEDB_URI: &str = "http://localhost:3001";
const DB_NAME: &str = "aoi";

fn on_connected(creds: &Credentials, _client_address: Address) {
    println!("{creds:?} {_client_address:?}");
}
/// Register all the callbacks our app will use to respond to database events.
fn register_callbacks() {
    // When we receive our `Credentials`, save them to a file.
    once_on_connect(on_connected);
}

fn main() {
    // register_callbacks();
    // connect(SPACETIMEDB_URI, DB_NAME, None).expect("Failed to connect");
    // loop {}
    // return;
    let args = Args::parse();
    let next_state = match args.mode {
        RunMode::CustomBattle => GameState::Battle,
        RunMode::Shop => GameState::Shop,
        RunMode::Test => GameState::TestsLoading,
    };
    let next_state = GameState::MainMenu;
    App::new()
        .add_state::<GameState>()
        .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.1)))
        .insert_resource(PkvStore::new("makscee", "arena_of_ideas"))
        .add_plugins((DefaultPlugins
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
            }),))
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
        .add_plugins(
            bevy_inspector_egui::quick::WorldInspectorPlugin::new()
                .run_if(input_toggle_active(false, KeyCode::Escape)),
        )
        .add_plugins(AudioPlugin)
        .add_plugins(Material2dPlugin::<LineShapeMaterial>::default())
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
            CustomBattlePlugin,
            PoolsPlugin,
            ActionPlugin,
            UnitPlugin,
            RepresentationPlugin,
            ShopPlugin,
            BattlePlugin,
            TestPlugin,
        ))
        // .add_systems(Update, ui_example_system)
        .add_systems(Startup, setup)
        .add_systems(Update, (input, input_world))
        .init_resource::<UserName>()
        .init_resource::<Password>()
        .init_resource::<GameTimer>()
        .register_type::<VarState>()
        .register_type::<VarStateDelta>()
        .add_systems(Update, show_build_version)
        .run();
}

fn setup(mut commands: Commands) {
    let mut camera = Camera2dBundle::default();
    camera.projection.scaling_mode = ScalingMode::FixedVertical(15.0);
    commands.spawn((camera, RaycastPickCamera::default()));
}

fn update(mut timer: ResMut<GameTimer>, time: Res<Time>) {
    timer.advance(time.delta_seconds());
}

fn input(
    input: Res<Input<KeyCode>>,
    mut timer: ResMut<GameTimer>,
    mut state: ResMut<NextState<GameState>>,
) {
    if input.just_pressed(KeyCode::Space) {
        let paused = timer.paused();
        timer.pause(!paused);
    }
    if input.just_pressed(KeyCode::R) {
        timer.reset();
        state.set(GameState::Restart);
    }
    if input.just_pressed(KeyCode::T) {
        timer.reset();
        state.set(GameState::TestsLoading);
    }
}

fn input_world(world: &mut World) {
    let input = world.get_resource::<Input<KeyCode>>().unwrap();
    if input.just_pressed(KeyCode::C) {
        UnitPlugin::clear_world(world);
        let battle = Options::get_custom_battle(world);
        let left = battle.left.clone();
        let right = battle.right.clone();
        dbg!(SimulationPlugin::run(left, right, world));
        UnitPlugin::clear_world(world);
    } else if input.just_pressed(KeyCode::S) {
        Save::default().save(world).unwrap();
        UnitPlugin::despawn_all(world);
        GameState::change(GameState::Restart, world);
    }
}

fn detect_changes(
    mut unit_events: EventReader<AssetEvent<PackedUnit>>,
    mut rep_events: EventReader<AssetEvent<Representation>>,
    mut battle_state_events: EventReader<AssetEvent<CustomBattleData>>,
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
    }) {
        state.set(GameState::Restart)
    }
}

const VERSION: &str = env!("CARGO_PKG_VERSION");
fn show_build_version(world: &mut World) {
    let ctx = &egui_context(world);
    Area::new("build version")
        .anchor(Align2::LEFT_BOTTOM, [10.0, -10.0])
        .show(ctx, |ui| ui.label(format!("arena-of-ideas {VERSION}")));
}
