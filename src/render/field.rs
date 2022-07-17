use super::*;

impl Render {
    pub fn draw_field(
        &self,
        shader_render: &ShaderConfig,
        game_time: f32,
        framebuffer: &mut ugli::Framebuffer,
    ) {
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
        let window_size = self.geng.window().size();

        ugli::draw(
            framebuffer,
            &shader_render.shader,
            ugli::DrawMode::TriangleFan,
            &quad,
            (
                ugli::uniforms! {
                    u_time: game_time,
                    u_unit_position: vec2(0.0,0.0),
                    u_window_size: window_size,
                },
                geng::camera2d_uniforms(&self.camera, framebuffer_size.map(|x| x as f32)),
                &shader_render.parameters,
            ),
            ugli::DrawParameters {
                blend_mode: Some(default()),
                ..default()
            },
        );
    }
}
