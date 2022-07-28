use super::*;

impl Render {
    pub fn draw_field(
        &self,
        shader_program: &ShaderProgram,
        game_time: f64,
        model: &Model,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let mut instances_arr: ugli::VertexBuffer<Instance> =
            ugli::VertexBuffer::new_dynamic(self.geng.ugli(), Vec::new());
        instances_arr.resize(shader_program.instances, Instance {});
        let quad = shader_program.get_vertices(&self.geng);
        let framebuffer_size = framebuffer.size();
        let window_size = self.geng.window().size();

        ugli::draw(
            framebuffer,
            &shader_program.program,
            ugli::DrawMode::TriangleStrip,
            ugli::instanced(&quad, &instances_arr),
            (
                ugli::uniforms! {
                    u_time: game_time,
                    u_window_size: window_size,
                    u_last_player_action_time: model.last_player_action_time.as_f32(),
                    u_last_enemy_action_time: model.last_enemy_action_time.as_f32(),
                },
                geng::camera2d_uniforms(&self.camera, framebuffer_size.map(|x| x as f32)),
                &shader_program.parameters,
            ),
            ugli::DrawParameters {
                blend_mode: Some(default()),
                ..default()
            },
        );
    }
}
