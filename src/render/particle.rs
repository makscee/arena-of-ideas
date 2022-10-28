use super::*;

impl Render {
    pub(super) fn draw_particle(
        &self,
        particle: &Particle,
        shader_program: &ShaderProgram,
        game_time: f64,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        if !particle.visible {
            return;
        }
        let mut instances_arr: ugli::VertexBuffer<Instance> =
            ugli::VertexBuffer::new_dynamic(self.geng.ugli(), Vec::new());
        instances_arr.resize(shader_program.instances, Instance {});
        let quad = shader_program.get_vertices(&self.geng);
        let framebuffer_size = framebuffer.size();
        ugli::draw(
            framebuffer,
            &shader_program.program,
            ugli::DrawMode::TriangleStrip,
            ugli::instanced(&quad, &instances_arr),
            (
                ugli::uniforms! {
                    u_time: game_time,
                    u_unit_position: particle.position.map(|x| x.as_f32()),
                    u_unit_radius: particle.radius.as_f32(),
                    u_spawn: (particle.time_left / particle.duration).as_f32(),
                    u_action: 0.0,
                    u_clan_color_1: Rgba::WHITE,
                    u_clan_color_2: Rgba::WHITE,
                    u_clan_color_3: Rgba::WHITE,
                    u_clan_count: 0,
                },
                geng::camera2d_uniforms(&self.camera, framebuffer_size.map(|x| x as f32)),
                shader_program.parameters.clone(),
            ),
            ugli::DrawParameters {
                blend_mode: Some(ugli::BlendMode::default()),
                ..default()
            },
        );
    }
}
