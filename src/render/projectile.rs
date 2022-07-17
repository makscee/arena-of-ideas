use super::*;

impl Render {
    pub fn draw_projectile(
        &self,
        projectile: &Projectile,
        render_mode: &RenderMode,
        game_time: f32,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        const RADIUS: f32 = 0.35;
        match render_mode {
            RenderMode::Circle { color } => {
                self.geng.draw_2d(
                    framebuffer,
                    &self.camera,
                    &draw_2d::Ellipse::circle(
                        projectile.position.map(|x| x.as_f32()),
                        RADIUS,
                        *color,
                    ),
                );
            }
            RenderMode::Texture { texture } => {
                self.geng.draw_2d(
                    framebuffer,
                    &self.camera,
                    &draw_2d::TexturedQuad::unit(&**texture)
                        .scale_uniform(RADIUS)
                        .translate(projectile.position.map(|x| x.as_f32())),
                );
            }
            RenderMode::Shader {
                program,
                parameters,
                vertices,
                ..
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
                let model_matrix = Mat3::translate(projectile.position.map(|x| x.as_f32()))
                    * Mat3::scale_uniform(RADIUS);
                let velocity = ((projectile.target_position - projectile.position)
                    .normalize_or_zero()
                    * projectile.speed)
                    .map(|x| x.as_f32());

                ugli::draw(
                    framebuffer,
                    program,
                    ugli::DrawMode::TriangleFan,
                    &quad,
                    (
                        ugli::uniforms! {
                            u_time: game_time,
                            u_unit_position: projectile.position.map(|x| x.as_f32()),
                            u_unit_radius: RADIUS,
                            u_velocity: velocity,
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
