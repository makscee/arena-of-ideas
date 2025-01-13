use super::*;

pub struct HeroPlugin;

impl Plugin for HeroPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, Self::setup)
            .add_systems(Update, (Self::update, Self::update_mover));
    }
}

impl HeroPlugin {
    fn setup(mut commands: Commands) {
        let entity = commands.spawn_empty().id();
        Hero {
            name: "HeroName".into(),
            ..default()
        }
        .unpack(entity, &mut commands);
        commands.entity(entity).insert(Mover::default());
    }
    fn update(
        mut ctx: Query<&mut EguiContext>,
        mut mover: Query<&mut Mover, With<Hero>>,
        cam: Query<(&Camera, &GlobalTransform)>,
    ) {
        let ctx = &ctx.single_mut().into_inner().get_mut().clone();
        if let Some(pos) = ctx.bg_clicked() {
            let (cam, cam_transform) = cam.single();
            let pos = screen_to_world_cam(pos.to_bvec2(), cam, cam_transform);
            let mut mover = mover.single_mut();
            mover.from = mover.pos(global_settings().hero_speed);
            mover.start_ts = now_seconds();
            mover.target = pos;
        }
    }
    fn update_mover(mut q: Query<(&mut Transform, &Mover)>) {
        let speed = global_settings().hero_speed;
        for (mut transform, m) in q.iter_mut() {
            transform.translation = m.pos(speed).extend(transform.translation.z);
        }
    }
}
