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
                self.geng.draw_2d(
                    framebuffer,
                    &self.camera,
                    &draw_2d::Ellipse::circle(
                        particle.position.map(|x| x.as_f32()),
                        particle.radius.as_f32(),
                        *color,
                    ),
                );
            }
            RenderMode::Texture { texture } => {
                self.geng.draw_2d(
                    framebuffer,
                    &self.camera,
                    &draw_2d::TexturedQuad::unit(&**texture)
                        .scale_uniform(particle.radius.as_f32())
                        .translate(particle.position.map(|x| x.as_f32())),
                );
            }
            RenderMode::Shader {
                program,
                parameters,
                vertices,
                instances,
            } => {
                let vert_count = *vertices;
                let mut vertices = vec![draw_2d::Vertex {
                    a_pos: vec2(-1.0, -1.0),
                }];
                for i in 0..vert_count {
                    vertices.push(draw_2d::Vertex {
                        a_pos: vec2((i as f32 / vert_count as f32) * 2.0 - 1.0, 1.0),
                    });
                    vertices.push(draw_2d::Vertex {
                        a_pos: vec2(((i + 1) as f32 / vert_count as f32) * 2.0 - 1.0, -1.0),
                    });
                }

                vertices.push(draw_2d::Vertex {
                    a_pos: vec2(1.0, 1.0),
                });

                let mut instances_arr: ugli::VertexBuffer<Instance> =
                    ugli::VertexBuffer::new_dynamic(self.geng.ugli(), Vec::new());
                instances_arr.resize(*instances, Instance {});
                let quad = ugli::VertexBuffer::new_dynamic(self.geng.ugli(), vertices);
                let framebuffer_size = framebuffer.size();
                ugli::draw(
                    framebuffer,
                    program,
                    ugli::DrawMode::TriangleStrip,
                    ugli::instanced(&quad, &instances_arr),
                    (
                        ugli::uniforms! {
                            u_time: game_time,
                            u_unit_position: particle.position.map(|x| x.as_f32()),
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
