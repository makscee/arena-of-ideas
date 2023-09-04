mod components;
mod login_menu_system;
mod materials;
mod plugins;
pub mod resourses;
mod utils;

use std::time::Duration;

use anyhow::Context as _;
use anyhow::{anyhow, Result};
use bevy::{
    asset::ChangeWatcher,
    input::common_conditions::input_toggle_active,
    log::LogPlugin,
    math::{vec2, vec3},
    prelude::*,
    reflect::{TypePath, TypeUuid},
    render::{camera::ScalingMode, render_resource::AsBindGroup},
    sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle},
    utils::*,
};
use bevy_asset_loader::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_egui::{
    egui::{CentralPanel, TextEdit},
    EguiContexts,
};
use bevy_mod_picking::prelude::*;
use components::*;
use itertools::Itertools;
use login_menu_system::*;
use materials::*;
use plugins::*;
use resourses::*;
use serde::*;
use utils::*;

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
        .add_systems(PreUpdate, update)
        .add_systems(PostUpdate, detect_changes)
        .add_collection_to_loading_state::<_, Options>(GameState::AssetLoading)
        .add_collection_to_loading_state::<_, Pools>(GameState::AssetLoading)
        .add_plugins(DefaultPickingPlugins)
        .add_plugins(
            bevy_inspector_egui::quick::WorldInspectorPlugin::new()
                .run_if(input_toggle_active(false, KeyCode::Escape)),
        )
        .add_plugins(Material2dPlugin::<LineShapeMaterial>::default())
        .add_plugins(RonAssetPlugin::<PackedUnit>::new(&["unit.ron"]))
        .add_plugins(RonAssetPlugin::<BattleState>::new(&["battle.ron"]))
        .add_plugins(RonAssetPlugin::<Representation>::new(&["rep.ron"]))
        .add_plugins(RonAssetPlugin::<Animations>::new(&["anim.ron"]))
        .add_plugins(RonAssetPlugin::<Statuses>::new(&["statuses.ron"]))
        .add_plugins((
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
    mut time: ResMut<Time>,
    mut state: ResMut<NextState<GameState>>,
) {
    if input.just_pressed(KeyCode::Space) {
        if time.is_paused() {
            time.unpause()
        } else {
            time.pause()
        }
    }
    if input.just_pressed(KeyCode::R) {
        state.set(GameState::Restart);
    }
}

fn detect_changes(
    mut unit_events: EventReader<AssetEvent<PackedUnit>>,
    mut rep_events: EventReader<AssetEvent<Representation>>,
    mut state: ResMut<NextState<GameState>>,
) {
    if unit_events.into_iter().any(|x| match x {
        AssetEvent::Modified { .. } => true,
        _ => false,
    }) || rep_events.into_iter().any(|x| match x {
        AssetEvent::Modified { .. } => true,
        _ => false,
    }) {
        state.set(GameState::Restart)
    }
}
