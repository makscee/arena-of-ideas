mod components;
mod materials;
mod module_bindings;
mod plugins;
mod prelude;
pub mod resources;
mod utils;

use bevy::render::camera::ClearColor;
use noisy_bevy::NoisyShaderPlugin;
pub use prelude::*;

#[derive(Parser, Debug, Default)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(short, long)]
    mode: RunMode,
    #[arg(short, long)]
    path: Option<String>,
}

#[derive(Debug, Clone, ValueEnum, Default)]
pub enum RunMode {
    #[default]
    Regular,
    Offline,
    Test,
    Custom,
    Last,
    Continue,
    Sync,
    Archive,
    Upload,
    Editor,
    Table,
    Gallery,
}

fn main() {
    let args = Args::try_parse().unwrap_or_default();
    let next_state = match args.mode {
        RunMode::Regular
        | RunMode::Offline
        | RunMode::Archive
        | RunMode::Upload
        | RunMode::Sync => GameState::MainMenu,
        RunMode::Custom => GameState::CustomBattle,
        RunMode::Gallery => GameState::HeroGallery,
        RunMode::Last => GameState::LastBattle,
        RunMode::Editor => GameState::HeroEditor,
        RunMode::Table => GameState::HeroTable,
        RunMode::Continue => GameState::Shop,
        RunMode::Test => GameState::TestsLoading,
    };
    match args.mode {
        RunMode::Sync => set_after_login_state(GameState::AssetSync),
        RunMode::Archive => set_after_login_state(GameState::ArenaArchiveSave),
        RunMode::Upload => set_after_login_state(GameState::ArenaArchiveUpload),
        _ => {}
    }
    let mut default_plugins = DefaultPlugins.set(LogPlugin {
        level: bevy::log::Level::DEBUG,
        filter: "info,debug,wgpu_core=warn,wgpu_hal=warn,naga=warn".into(),
        ..default()
    });
    match args.mode {
        RunMode::Regular
        | RunMode::Offline
        | RunMode::Custom
        | RunMode::Gallery
        | RunMode::Last
        | RunMode::Continue
        | RunMode::Editor
        | RunMode::Table => {
            default_plugins = default_plugins.set(bevy::window::WindowPlugin {
                primary_window: Some(bevy::prelude::Window {
                    title: "Arena of Ideas".into(),
                    ..default()
                }),
                ..default()
            })
        }
        RunMode::Test | RunMode::Sync | RunMode::Archive | RunMode::Upload => {
            default_plugins = default_plugins.set(bevy::window::WindowPlugin {
                primary_window: None,
                exit_condition: bevy::window::ExitCondition::DontExit,
                ..default()
            })
        }
    };
    if matches!(args.mode, RunMode::Offline) {
        set_offline(true);
    }
    let mut app = App::new();
    app.init_state::<GameState>()
        .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.1)))
        .insert_resource(PkvStore::new("makscee", "arena_of_ideas"))
        .add_plugins((default_plugins, FrameTimeDiagnosticsPlugin))
        .add_loading_state(
            LoadingState::new(GameState::Loading)
                .continue_to_state(next_state)
                .load_collection::<Options>()
                .load_collection::<Pools>()
                .with_dynamic_assets_file::<StandardDynamicAssetCollection>(
                    "ron/_dynamic.assets.ron",
                ),
        )
        .add_loading_state(
            LoadingState::new(GameState::TestsLoading)
                .continue_to_state(GameState::BattleTest)
                .load_collection::<TestScenarios>(),
        )
        .add_systems(PreUpdate, update)
        .add_systems(PostUpdate, detect_changes)
        .add_plugins(bevy_egui::EguiPlugin)
        .add_plugins(DefaultPickingPlugins)
        .add_plugins(bevy_kira_audio::AudioPlugin)
        .add_plugins(NoisyShaderPlugin)
        .add_plugins(Material2dPlugin::<ShapeMaterial>::default())
        .add_plugins(Material2dPlugin::<CurveMaterial>::default())
        .add_plugins(RonAssetPlugin::<PackedUnit>::new(&["unit.ron"]))
        .add_plugins(RonAssetPlugin::<House>::new(&["house.ron"]))
        .add_plugins(RonAssetPlugin::<CustomBattleData>::new(&["battle.ron"]))
        .add_plugins(RonAssetPlugin::<Representation>::new(&["rep.ron"]))
        .add_plugins(RonAssetPlugin::<OptionsData>::new(&["options.ron"]))
        .add_plugins(RonAssetPlugin::<Animations>::new(&["anim.ron"]))
        .add_plugins(RonAssetPlugin::<TestScenario>::new(&["scenario.ron"]))
        .add_plugins(RonAssetPlugin::<Vfx>::new(&["vfx.ron"]))
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
        .add_plugins((
            LoginPlugin,
            ProfilePlugin,
            PanelsPlugin,
            UiPlugin,
            AlertPlugin,
            AssetsUploadPlugin,
            OperationsPlugin,
            TeamPlugin,
            ArenaArchivePlugin,
            HeroTablePlugin,
        ))
        .add_systems(Update, input_world)
        .register_type::<VarState>()
        .register_type::<VarStateDelta>();
    app.add_systems(Startup, setup);
    match args.mode {
        RunMode::Regular
        | RunMode::Offline
        | RunMode::Continue
        | RunMode::Last
        | RunMode::Sync
        | RunMode::Archive
        | RunMode::Upload => {
            app.add_systems(OnExit(GameState::Loading), LoginPlugin::setup);
        }
        RunMode::Test | RunMode::Custom | RunMode::Gallery | RunMode::Editor | RunMode::Table => {}
    }

    app.run();
}

fn setup(world: &mut World) {
    if let Some(ctx) = egui_context(world) {
        egui_extras::install_image_loaders(&ctx);
    }
}

fn update(time: Res<Time>, audio: Res<AudioData>) {
    let mut timer = GameTimer::get();
    if let Some(play_delta) = audio.play_delta {
        timer.advance_play(play_delta);
    } else {
        timer.advance_play(time.delta_seconds());
    }
}

fn input_world(world: &mut World) {
    let input = world.get_resource::<ButtonInput<KeyCode>>().unwrap();
    if !input.pressed(KeyCode::ControlLeft) {
        return;
    }
    if input.just_pressed(KeyCode::KeyR) {
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
    if unit_events
        .read()
        .any(|x| matches!(x, AssetEvent::Modified { .. }))
        || rep_events
            .read()
            .any(|x| matches!(x, AssetEvent::Modified { .. }))
        || battle_state_events
            .read()
            .any(|x| matches!(x, AssetEvent::Modified { .. }))
        || vfx_events
            .read()
            .any(|x| matches!(x, AssetEvent::Modified { .. }))
    {
        state.set(GameState::Loading)
    }
}

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
