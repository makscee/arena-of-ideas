use super::*;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Loading), Self::respawn_camera)
            .add_systems(
                Update,
                Self::adjust_to_fit_units
                    .run_if(in_state(GameState::Battle).or_else(in_state(GameState::Shop))),
            );
    }
}

#[derive(Resource)]
pub struct CameraData {
    pub entity: Entity,
    pub need_scale: f32,
    pub cur_scale: f32,
}

const SCALE_CHANGE_SPEED: f32 = 3.0;

impl CameraPlugin {
    pub fn cursor_world_pos(world: &mut World) -> Option<Vec2> {
        if let Some(cursor_pos) = cursor_pos(world) {
            let cam = world.resource::<CameraData>().entity;
            Some(screen_to_world(
                cursor_pos,
                world.get::<Camera>(cam).unwrap(),
                world.get::<GlobalTransform>(cam).unwrap(),
            ))
        } else {
            None
        }
    }

    fn respawn_camera(mut commands: Commands, data: Option<ResMut<CameraData>>) {
        let mut camera = Camera2dBundle::default();
        camera.projection.scaling_mode = ScalingMode::FixedVertical(15.0);
        let entity = commands.spawn((camera, RaycastPickCamera::default())).id();
        if let Some(data) = data {
            commands.entity(data.entity).despawn_recursive();
        }
        let data = CameraData {
            entity,
            need_scale: default(),
            cur_scale: 100.0,
        };
        commands.insert_resource(data);
    }

    fn adjust_to_fit_units(
        visible: Query<(&Transform, &ComputedVisibility)>,
        mut camera: Query<&mut OrthographicProjection>,
        mut data: ResMut<CameraData>,
        time: Res<Time>,
    ) {
        let mut camera = camera.single_mut();
        let mut width = 15.0_f32;
        for (t, cv) in visible.iter() {
            if cv.is_visible_in_hierarchy() {
                width = width.max((t.translation.x.abs() + 1.5) * 2.0);
            }
        }
        data.need_scale = width;
        data.cur_scale +=
            (data.need_scale - data.cur_scale) * time.delta_seconds() * SCALE_CHANGE_SPEED;

        camera.scaling_mode = ScalingMode::FixedHorizontal(data.cur_scale);
    }
}
