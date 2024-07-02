use bevy::{
    core_pipeline::core_2d::Camera2dBundle,
    ecs::system::Commands,
    render::{
        camera::{OrthographicProjection, ScalingMode},
        view::InheritedVisibility,
    },
    window::PrimaryWindow,
};

use super::*;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Title), Self::respawn_camera)
            .add_systems(OnEnter(GameState::Loading), Self::respawn_camera)
            .add_systems(
                Update,
                Self::adjust_to_fit_units.run_if(in_state(GameState::Battle)),
            );
    }
}

#[derive(Resource)]
pub struct CameraData {
    pub entity: Entity,
    pub need_scale: f32,
    pub cur_scale: f32,
    pub slot_pixel_spacing: f32,
}

const SCALE_CHANGE_SPEED: f32 = 3.0;
pub const SLOT_SPACING: f32 = 3.0;

impl CameraPlugin {
    pub fn entity(world: &World) -> Entity {
        world.resource::<CameraData>().entity
    }
    pub fn get_entity(world: &World) -> Option<Entity> {
        world.get_resource::<CameraData>().map(|cd| cd.entity)
    }
    pub fn cursor_world_pos(world: &mut World) -> Option<Vec2> {
        if let Some(cursor_pos) = cursor_pos(world) {
            Some(screen_to_world(cursor_pos, world))
        } else {
            None
        }
    }
    fn respawn_camera(mut commands: Commands, data: Option<ResMut<CameraData>>) {
        let mut camera = Camera2dBundle::default();
        camera.projection.scaling_mode = ScalingMode::FixedVertical(15.0);
        let entity = commands.spawn(camera).id();
        if let Some(data) = data {
            commands.entity(data.entity).despawn_recursive();
        }
        let data = CameraData {
            entity,
            cur_scale: 100.0,
            slot_pixel_spacing: 150.0,
            need_scale: default(),
        };
        commands.insert_resource(data);
    }
    fn adjust_to_fit_units(
        visible: Query<(&Transform, &InheritedVisibility)>,
        mut projection: Query<(&mut OrthographicProjection, &Camera)>,
        mut data: ResMut<CameraData>,
        window: Query<&bevy::window::Window, With<PrimaryWindow>>,
        time: Res<Time>,
    ) {
        let (mut projection, camera) = projection.single_mut();
        let mut width: f32 = 28.0;
        let aspect_ratio = camera
            .logical_target_size()
            .map(|v| v.x / v.y)
            .unwrap_or(1.0);
        for (t, iv) in visible.iter() {
            if iv.get() {
                width = width
                    .max((t.translation.x.abs() + 1.5) * 2.0)
                    .max(((t.translation.y.abs() + 2.0) * aspect_ratio) * 2.0);
            }
        }
        data.need_scale = width;
        data.cur_scale +=
            (data.need_scale - data.cur_scale) * time.delta_seconds() * SCALE_CHANGE_SPEED;
        let window_width = window.single().resolution.width();
        data.slot_pixel_spacing = SLOT_SPACING / data.cur_scale * window_width;

        projection.scaling_mode = ScalingMode::FixedHorizontal(data.cur_scale);
    }
}
