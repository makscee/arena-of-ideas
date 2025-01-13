use egui::LayerId;

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
        let mut mover = mover.single_mut();
        let (cam, cam_transform) = cam.single();
        let speed = global_settings().hero_speed;
        if let Some(pos) = ctx.bg_clicked() {
            let pos = screen_to_world_cam(pos.to_bvec2(), cam, cam_transform);
            mover.from = mover.pos(speed);
            mover.start_ts = now_seconds();
            mover.target = pos;
        }
        let pos = world_to_screen_cam(mover.pos(speed).extend(0.0), cam, cam_transform);
        let target = world_to_screen_cam(mover.target.extend(0.0), cam, cam_transform);
        let dir = target - pos;
        if dir.length() > 0.01 {
            let p = ctx.layer_painter(LayerId::background());
            let target = target.to_pos2();
            p.line_segment(
                [
                    (pos + dir.normalize() * UNIT_SIZE * unit_pixels() * 1.1).to_pos2(),
                    target,
                ],
                STROKE_DARK,
            );
            let t = (2.0 - mover.t(speed) * 5.0).at_least(1.0);
            let t = t * t;
            p.circle_filled(target, 13.0 * t, VISIBLE_DARK);
            p.circle_stroke(target, 20.0 * t, STROKE_DARK);
        }
    }
    fn update_mover(mut q: Query<(&mut Transform, &Mover)>) {
        let speed = global_settings().hero_speed;
        for (mut transform, m) in q.iter_mut() {
            transform.translation = m.pos(speed).extend(transform.translation.z);
        }
    }
}
