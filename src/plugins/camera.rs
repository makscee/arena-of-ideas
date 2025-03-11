use bevy::{
    core_pipeline::core_2d::Camera2dBundle,
    render::camera::{OrthographicProjection, ScalingMode},
};

use super::*;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, Self::update);
    }
}

#[derive(Resource, Clone, Copy)]
pub struct CameraData {
    pub entity: Entity,
    pub need_scale: f32,
    pub cur_scale: f32,
}

const SCALE_CHANGE_SPEED: f32 = 3.0;
pub const SLOT_SPACING: f32 = 3.0;

impl CameraData {
    pub fn apply(&self, projection: &mut OrthographicProjection) {
        projection.scaling_mode = ScalingMode::FixedHorizontal {
            viewport_width: self.cur_scale,
        };
    }
}

impl CameraPlugin {
    fn update(
        mut cam: Query<(&mut Transform, &mut OrthographicProjection, &Camera)>,
        mut ctx: Query<&mut EguiContext>,
        mut data: ResMut<CameraData>,
    ) {
        let ctx = ctx.single_mut().into_inner().get_mut();
        let mut cam = cam.single_mut();
        data.cur_scale +=
            (data.need_scale - data.cur_scale) * gt().last_delta() * SCALE_CHANGE_SPEED;
        data.apply(&mut cam.1);
        set_unit_pixels(ctx.screen_rect().width() / data.cur_scale);
    }
    pub fn apply(world: &mut World) {
        let cd = *world.resource::<CameraData>();
        if let Some(mut proj) = world.get_mut::<OrthographicProjection>(Self::entity(world)) {
            cd.apply(&mut proj);
        }
    }
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
    pub fn respawn_camera(world: &mut World) {
        let mut camera = Camera2dBundle::default();
        let entity = world.spawn_empty().id();
        let data = CameraData {
            entity,
            cur_scale: 25.0,
            need_scale: 25.0,
        };
        data.apply(&mut camera.projection);
        if let Some(data) = world.get_resource::<CameraData>() {
            world.entity_mut(data.entity).despawn_recursive();
        }
        world.entity_mut(entity).insert(camera);
        world.insert_resource(data);
    }
}
