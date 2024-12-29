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
        return;
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
            mover.single_mut().need_pos = pos;
        }
    }
    fn update_mover(mut q: Query<(&mut Transform, &Mover)>) {
        for (mut t, m) in q.iter_mut() {
            let delta = (m.need_pos - t.translation.xy()) * gt().last_delta() * 5.0;
            t.translation += delta.extend(0.0);
        }
    }
}

#[derive(Component, Default)]
struct Mover {
    need_pos: Vec2,
}
