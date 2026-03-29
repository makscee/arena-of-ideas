use bevy::prelude::*;
use client::plugins::{
    collection::CollectionPlugin, connect::ConnectPlugin, create::CreatePlugin, game::GamePlugin,
    incubator::IncubatorPlugin, ui::UiPlugin,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Arena of Ideas".to_string(),
                resolution: bevy::window::WindowResolution::new(1280, 720),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(UiPlugin)
        .add_plugins(GamePlugin)
        .add_plugins(ConnectPlugin)
        .add_plugins(CollectionPlugin)
        .add_plugins(CreatePlugin)
        .add_plugins(IncubatorPlugin)
        .add_systems(Startup, setup_camera)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
