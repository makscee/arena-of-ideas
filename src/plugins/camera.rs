use bevy::{
    core_pipeline::core_2d::Camera2dBundle,
    prelude::Without,
    render::{
        camera::{OrthographicProjection, ScalingMode},
        view::InheritedVisibility,
    },
};

use super::*;

static UNIT_PIXELS: Mutex<f32> = Mutex::new(100.0);
pub fn unit_pixels() -> f32 {
    *UNIT_PIXELS.lock().unwrap()
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            Self::adjust_to_fit_units.run_if(in_state(GameState::Battle)),
        )
        .add_systems(Update, Self::update);
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
        projection.scaling_mode = ScalingMode::FixedHorizontal(self.cur_scale);
    }
}

impl CameraPlugin {
    fn update(
        mut cam: Query<(&mut Transform, &Camera), Without<Hero>>,
        mut ctx: Query<&mut EguiContext>,
        hero: Query<&Transform, With<Hero>>,
        data: Res<CameraData>,
    ) {
        let ctx = ctx.single_mut().into_inner().get_mut();
        let mut cam = cam.single_mut();
        cam.0.translation = hero.single().translation;
        *UNIT_PIXELS.lock().unwrap() = ctx.screen_rect().width() / data.cur_scale;
    }
    pub fn apply(world: &mut World) {
        let cd = *world.resource::<CameraData>();
        if let Some(mut proj) = world.get_mut::<OrthographicProjection>(Self::entity(world)) {
            cd.apply(&mut proj);
        }
    }
    pub fn pixel_unit(ctx: &egui::Context, world: &World) -> f32 {
        let width = ctx.screen_rect().width();
        width / world.resource::<CameraData>().cur_scale
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
            need_scale: default(),
        };
        data.apply(&mut camera.projection);
        if let Some(data) = world.get_resource::<CameraData>() {
            world.entity_mut(data.entity).despawn_recursive();
        }
        world.entity_mut(entity).insert(camera);
        world.insert_resource(data);
    }
    fn adjust_to_fit_units(
        visible: Query<(&Transform, &InheritedVisibility)>,
        mut projection: Query<(&mut OrthographicProjection, &Camera)>,
        mut data: ResMut<CameraData>,
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
        data.apply(&mut projection);
    }
}
