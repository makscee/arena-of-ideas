mod components;
mod login_menu_system;
mod materials;
mod plugins;
mod resourses;

use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use bevy::{
    asset::ChangeWatcher,
    log::LogPlugin,
    math::{vec2, vec3},
    prelude::*,
    reflect::{TypePath, TypeUuid},
    render::{camera::ScalingMode, render_resource::AsBindGroup},
    sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle},
    utils::HashMap,
};
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_egui::{
    egui::{CentralPanel, TextEdit},
    EguiContexts, EguiPlugin,
};
use components::*;
use itertools::Itertools;
use log::debug;
use login_menu_system::*;
use materials::*;
use plugins::*;
use resourses::*;
use serde::*;

fn main() {
    App::new()
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
        .add_plugins(bevy_inspector_egui::quick::WorldInspectorPlugin::new())
        .add_plugins(Material2dPlugin::<SdfShapeMaterial>::default())
        .add_plugins(RonAssetPlugin::<PackedUnit>::new(&["unit.ron"]))
        .add_plugins((UnitPlugin, RepresentationPlugin))
        // .add_plugins(EguiPlugin)
        // .add_systems(Update, ui_example_system)
        .add_systems(Startup, (setup_units, setup))
        .add_systems(Update, spawn_units.run_if(run_once()))
        .add_systems(Update, input)
        .init_resource::<UserName>()
        .init_resource::<Password>()
        .run();
}

fn setup_units(mut commands: Commands, asset_server: Res<AssetServer>) {
    let unit = UnitHandle(asset_server.load("ron/1.unit.ron"));
    commands.insert_resource(unit);
}

fn spawn_units(world: &mut World) {
    debug!("Spawn units");
    let units = world
        .get_resource::<Assets<PackedUnit>>()
        .unwrap()
        .iter()
        .map(|(_, x)| x.clone())
        .collect_vec();
    for unit in units {
        dbg!(unit).unpack(world);
    }
}

fn setup(mut commands: Commands) {
    let mut camera = Camera2dBundle::default();
    camera.projection.scaling_mode = ScalingMode::FixedVertical(15.0);
    commands.spawn(camera);
}

fn input(input: Res<Input<KeyCode>>, mut time: ResMut<Time>) {
    if input.just_pressed(KeyCode::Space) {
        if time.is_paused() {
            time.unpause()
        } else {
            time.pause()
        }
    }
}
