use super::*;

impl Render {
    pub(super) fn draw_particle(
        &self,
        particle: &Particle,
        render_mode: &RenderMode,
        game_time: f32,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        match render_mode {
            RenderMode::Circle { color } => {
                let particle_pos = particle.position.map(|x| x.as_f32());
                self.geng.draw_2d(
                    framebuffer,
                    &self.camera,
                    &draw_2d::Ellipse::circle(particle_pos, particle.radius.as_f32(), *color),
                );
            }
            RenderMode::Texture { texture } => {
                let particle_pos = particle.position.map(|x| x.as_f32());
                self.geng.draw_2d(
                    framebuffer,
                    &self.camera,
                    &draw_2d::TexturedQuad::unit(&**texture)
                        .scale_uniform(particle.radius.as_f32())
                        .translate(particle_pos),
                );
            }
            RenderMode::Shader {
                program,
                parameters,
            } => {
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
                let particle_pos = particle.position.map(|x| x.as_f32());
                let model_matrix =
                    Mat3::translate(particle_pos) * Mat3::scale_uniform(particle.radius.as_f32());

                ugli::draw(
                    framebuffer,
                    program,
                    ugli::DrawMode::TriangleFan,
                    &quad,
                    (
                        ugli::uniforms! {
                            u_time: game_time,
                            u_unit_position: particle_pos,
                            u_unit_radius: particle.radius.as_f32(),
                            u_spawn: (particle.time_left / particle.duration).as_f32(),
                            u_action: 0.0,
                            u_clan_color_1: Color::WHITE,
                            u_clan_color_2: Color::WHITE,
                            u_clan_color_3: Color::WHITE,
                            u_clan_count: 0,
                        },
                        geng::camera2d_uniforms(&self.camera, framebuffer_size.map(|x| x as f32)),
                        parameters,
                    ),
                    ugli::DrawParameters {
                        blend_mode: Some(default()),
                        ..default()
                    },
                );
            }
        }
    }
}
