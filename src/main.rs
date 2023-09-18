mod components;
mod login_menu_system;
mod materials;
mod plugins;
mod prelude;
pub mod resourses;
mod utils;
use prelude::*;

fn main() {
    App::new()
        .add_state::<GameState>()
        .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.1)))
        .add_plugins((DefaultPlugins
            .set(AssetPlugin {
                watch_for_changes: ChangeWatcher::with_delay(Duration::from_millis(100)),
                ..default()
            })
            .set(LogPlugin {
                level: bevy::log::Level::DEBUG,
                filter: "info,debug,wgpu_core=warn,wgpu_hal=warn,naga=warn".into(),
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Arena of Ideas".into(),
                    ..default()
                }),
                ..default()
            }),))
        .add_loading_state(
            LoadingState::new(GameState::AssetLoading).continue_to_state(GameState::Shop),
        )
        .add_dynamic_collection_to_loading_state::<_, StandardDynamicAssetCollection>(
            GameState::AssetLoading,
            "ron/dynamic.assets.ron",
        )
        .add_collection_to_loading_state::<_, Options>(GameState::AssetLoading)
        .add_collection_to_loading_state::<_, Pools>(GameState::AssetLoading)
        .add_systems(PreUpdate, update)
        .add_systems(PostUpdate, detect_changes)
        .add_plugins(DefaultPickingPlugins)
        .add_plugins(
            bevy_inspector_egui::quick::WorldInspectorPlugin::new()
                .run_if(input_toggle_active(false, KeyCode::Escape)),
        )
        .add_plugins(Material2dPlugin::<LineShapeMaterial>::default())
        .add_plugins(RonAssetPlugin::<PackedUnit>::new(&["unit.ron"]))
        .add_plugins(RonAssetPlugin::<House>::new(&["house.ron"]))
        .add_plugins(RonAssetPlugin::<BattleState>::new(&["battle.ron"]))
        .add_plugins(RonAssetPlugin::<Representation>::new(&["rep.ron"]))
        .add_plugins(RonAssetPlugin::<Animations>::new(&["anim.ron"]))
        .add_plugins((
            PoolsPlugin,
            ActionPlugin,
            UnitPlugin,
            RepresentationPlugin,
            ShopPlugin,
            BattlePlugin,
        ))
        // .add_systems(Update, ui_example_system)
        .add_systems(Startup, setup)
        .add_systems(Update, input)
        .init_resource::<UserName>()
        .init_resource::<Password>()
        .init_resource::<GameTimer>()
        .register_type::<VarState>()
        .register_type::<VarStateDelta>()
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
}

fn detect_changes(
    mut unit_events: EventReader<AssetEvent<PackedUnit>>,
    mut rep_events: EventReader<AssetEvent<Representation>>,
    mut battle_state_events: EventReader<AssetEvent<BattleState>>,
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
