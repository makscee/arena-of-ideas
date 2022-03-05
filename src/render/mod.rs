use super::*;

impl Game {
    pub fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        ugli::clear(framebuffer, Some(Color::WHITE), None);
        for unit in itertools::chain![&self.units, &self.spawning_units] {
            self.geng.draw_2d(
                framebuffer,
                &self.camera,
                &draw_2d::Ellipse::circle(
                    unit.position.map(|x| x.as_f32()),
                    unit.radius().as_f32()
                        * match &unit.attack_state {
                            AttackState::Start { time, .. } => {
                                1.0 - 0.25 * (*time / unit.attack_animation_delay).as_f32()
                            }
                            _ => 1.0,
                        },
                    {
                        let mut color = unit.color;
                        if unit
                            .statuses
                            .iter()
                            .any(|status| matches!(status, Status::Freeze))
                        {
                            color = Color::CYAN;
                        }
                        if unit
                            .statuses
                            .iter()
                            .any(|status| matches!(status, Status::Slow { .. }))
                        {
                            color = Color::GRAY;
                        }
                        color
                    },
                ),
            );
            if unit
                .statuses
                .iter()
                .any(|status| matches!(status, Status::Shield))
            {
                self.geng.draw_2d(
                    framebuffer,
                    &self.camera,
                    &draw_2d::Ellipse::circle(
                        unit.position.map(|x| x.as_f32()),
                        unit.radius().as_f32() * 1.1,
                        Color::rgba(1.0, 1.0, 0.0, 0.5),
                    ),
                );
            }
            self.geng.draw_2d(
                framebuffer,
                &self.camera,
                &draw_2d::Quad::unit(Color::GREEN).transform(
                    Mat3::translate(unit.position.map(|x| x.as_f32()))
                        * Mat3::scale_uniform(unit.radius().as_f32())
                        * Mat3::translate(vec2(0.0, 1.2))
                        * Mat3::scale(
                            0.1 * vec2(10.0 * unit.hp.as_f32() / unit.max_hp.as_f32(), 1.0),
                        ),
                ),
            );
        }
        for projectile in &self.projectiles {
            self.geng.draw_2d(
                framebuffer,
                &self.camera,
                &draw_2d::Ellipse::circle(projectile.position.map(|x| x.as_f32()), 0.1, Color::RED),
            );
        }
    }
}
