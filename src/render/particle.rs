use super::*;

impl Render {
    pub(super) fn draw_particle(
        &self,
        particle: &Particle,
        shader_program: &ShaderProgram,
        game_time: f32,
        framebuffer: &mut ugli::Framebuffer,
    ) {
        let vert_count = shader_program.vertices;
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
        instances_arr.resize(shader_program.instances, Instance {});
        let quad = ugli::VertexBuffer::new_dynamic(self.geng.ugli(), vertices);
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
                    u_clan_color_1: Color::WHITE,
                    u_clan_color_2: Color::WHITE,
                    u_clan_color_3: Color::WHITE,
                    u_clan_count: 0,
                },
                geng::camera2d_uniforms(&self.camera, framebuffer_size.map(|x| x as f32)),
                shader_program.parameters.clone(),
            ),
            ugli::DrawParameters {
                blend_mode: Some(default()),
                ..default()
            },
        );
    }
}
