use bevy::{
    render::mesh::Mesh2d,
    sprite::{Material2dPlugin, MeshMaterial2d},
};

use super::*;

pub struct BackgroundPlugin;

impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, Self::setup)
            .add_plugins(Material2dPlugin::<BackgroundMaterial>::default());
    }
}

impl BackgroundPlugin {
    fn setup(
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<BackgroundMaterial>>,
    ) {
        commands.spawn((
            Mesh2d(meshes.add(Rectangle {
                half_size: vec2(1000.0, 1000.0),
            })),
            MeshMaterial2d(materials.add(BackgroundMaterial {})),
            Transform::from_translation(vec3(0.0, 0.0, -10.0)),
        ));
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct BackgroundMaterial {}

impl Material2d for BackgroundMaterial {
    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        "shaders/background.wgsl".into()
    }
}
