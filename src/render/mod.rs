use super::*;

struct Text {
    position: Vec2<f32>,
    velocity: Vec2<f32>,
    time: f32,
    text: String,
    color: Color<f32>,
}

pub struct Render {
    texts: Vec<Text>,
}

impl Render {
    pub fn new() -> Self {
        Self { texts: Vec::new() }
    }
    pub fn update(&mut self, delta_time: f32) {
        for text in &mut self.texts {
            text.time += delta_time;
            text.position += text.velocity * delta_time;
        }
        self.texts.retain(|text| text.time < 1.0);
    }
    pub fn add_text(&mut self, position: Vec2<Coord>, text: &str, color: Color<f32>) {
        let velocity = vec2(0.2, 0.0).rotate(global_rng().gen_range(0.0..2.0 * f32::PI));
        self.texts.push(Text {
            position: position.map(|x| x.as_f32()) + velocity,
            time: 0.0,
            velocity,
            text: text.to_owned(),
            color,
        });
    }
}

impl Game {
    pub fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        ugli::clear(framebuffer, Some(Color::WHITE), None);
        for unit in itertools::chain![&self.model.units, &self.model.spawning_units] {
            let template = &self.assets.units[&unit.unit_type];

            match &unit.render {
                RenderMode::Circle { color } => {
                    self.geng.draw_2d(
                        framebuffer,
                        &self.camera,
                        &draw_2d::Ellipse::circle(
                            unit.position.map(|x| x.as_f32()),
                            unit.radius().as_f32()
                                * match &unit.attack_state {
                                    AttackState::Start { time, .. } => {
                                        1.0 - 0.25 * (*time / unit.attack.animation_delay).as_f32()
                                    }
                                    _ => 1.0,
                                }
                                * match unit.spawn_animation_time_left {
                                    Some(time)
                                        if template.spawn_animation_time > Time::new(0.0) =>
                                    {
                                        1.0 - (time / template.spawn_animation_time).as_f32()
                                    }
                                    _ => 1.0,
                                },
                            {
                                let mut color = *color;
                                if unit
                                    .all_statuses
                                    .iter()
                                    .any(|status| matches!(status, Status::Freeze))
                                {
                                    color = Color::CYAN;
                                }
                                if unit
                                    .all_statuses
                                    .iter()
                                    .any(|status| matches!(status, Status::Slow { .. }))
                                {
                                    color = Color::GRAY;
                                }
                                color
                            },
                        ),
                    );
                }
                RenderMode::Texture { texture } => {
                    self.geng.draw_2d(
                        framebuffer,
                        &self.camera,
                        &draw_2d::TexturedQuad::unit(&**texture)
                            .scale_uniform(
                                unit.radius().as_f32()
                                    * match &unit.attack_state {
                                        AttackState::Start { time, .. } => {
                                            1.0 - 0.25
                                                * (*time / unit.attack.animation_delay).as_f32()
                                        }
                                        _ => 1.0,
                                    }
                                    * match unit.spawn_animation_time_left {
                                        Some(time)
                                            if template.spawn_animation_time > Time::new(0.0) =>
                                        {
                                            1.0 - (time / template.spawn_animation_time).as_f32()
                                        }
                                        _ => 1.0,
                                    },
                            )
                            .translate(unit.position.map(|x| x.as_f32())),
                    );
                }
                RenderMode::Shader { program } => {
                    let quad = ugli::VertexBuffer::new_dynamic(
                        self.geng.ugli(),
                        vec![
                            draw_2d::Vertex {
                                a_pos: vec2(-1.0, -1.0),
                            },
                            draw_2d::Vertex {
                                a_pos: vec2(1.0, -1.0),
                            },
                            draw_2d::Vertex {
                                a_pos: vec2(1.0, 1.0),
                            },
                            draw_2d::Vertex {
                                a_pos: vec2(-1.0, 1.0),
                            },
                        ],
                    );
                    let framebuffer_size = framebuffer.size();
                    let model_matrix = Mat3::translate(unit.position.map(|x| x.as_f32()))
                        * Mat3::scale_uniform(
                            unit.radius().as_f32()
                                * match &unit.attack_state {
                                    AttackState::Start { time, .. } => {
                                        1.0 - 0.25 * (*time / unit.attack.animation_delay).as_f32()
                                    }
                                    _ => 1.0,
                                }
                                * match unit.spawn_animation_time_left {
                                    Some(time)
                                        if template.spawn_animation_time > Time::new(0.0) =>
                                    {
                                        1.0 - (time / template.spawn_animation_time).as_f32()
                                    }
                                    _ => 1.0,
                                },
                        );
                    ugli::draw(
                        framebuffer,
                        program,
                        ugli::DrawMode::TriangleFan,
                        &quad,
                        (
                            ugli::uniforms! {
                                u_time: self.time,
                                u_unit_position: unit.position.map(|x| x.as_f32()),
                                u_unit_radius: unit.radius().as_f32(),
                                u_spawn: match unit.spawn_animation_time_left {
                                    Some(time)
                                        if template.spawn_animation_time > Time::new(0.0) =>
                                    {
                                        1.0 - (time / template.spawn_animation_time).as_f32()
                                    }
                                    _ => 1.0,
                                },
                                u_attack: match &unit.attack_state {
                                    AttackState::Start { time, .. } => {
                                        (*time / unit.attack.animation_delay).as_f32()
                                    }
                                    _ => 0.0,
                                },
                            },
                            geng::camera2d_uniforms(
                                &self.camera,
                                framebuffer_size.map(|x| x as f32),
                            ),
                        ),
                        ugli::DrawParameters {
                            blend_mode: Some(default()),
                            ..default()
                        },
                    );
                }
            }
            if unit
                .all_statuses
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
        for projectile in &self.model.projectiles {
            self.geng.draw_2d(
                framebuffer,
                &self.camera,
                &draw_2d::Ellipse::circle(projectile.position.map(|x| x.as_f32()), 0.1, Color::RED),
            );
        }
        for text in &self.render.texts {
            self.geng.draw_2d(
                framebuffer,
                &self.camera,
                &draw_2d::Text::unit(&**self.geng.default_font(), &text.text, text.color)
                    .scale_uniform(0.05)
                    .translate(text.position),
            );
        }
    }
}
