use super::*;

pub struct ViewRender {
    camera: Camera2d,
    geng: Geng,
    pub assets: Rc<Assets>,
}

impl ViewRender {
    pub fn new(camera: Camera2d, geng: Geng, assets: Rc<Assets>) -> Self {
        Self {
            camera,
            geng,
            assets,
        }
    }

    pub fn draw_shader<U>(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        shader_program: &ShaderProgram,
        uniforms: U,
    ) where
        U: Uniforms,
    {
        let mut instances_arr: ugli::VertexBuffer<Instance> =
            ugli::VertexBuffer::new_dynamic(self.geng.ugli(), Vec::new());
        instances_arr.resize(shader_program.instances, Instance {});
        let quad = shader_program.get_vertices(&self.geng);
        let framebuffer_size = framebuffer.size();

        let program = shader_program.program.as_ref();
        if let Some(program) = program {
            ugli::draw(
                framebuffer,
                &program,
                ugli::DrawMode::TriangleStrip,
                ugli::instanced(&quad, &instances_arr),
                (
                    ugli::uniforms! {
                        u_time: 0.0,
                    },
                    geng::camera2d_uniforms(&self.camera, framebuffer_size.map(|x| x as f32)),
                    &shader_program.parameters,
                    uniforms,
                ),
                ugli::DrawParameters {
                    blend_mode: Some(ugli::BlendMode::default()),
                    ..default()
                },
            );
        }
    }

    pub fn draw_unit(&self, framebuffer: &mut ugli::Framebuffer, unit_render: &UnitRender) {
        self.draw_shader(
            framebuffer,
            &self.assets.system_shaders.unit,
            uniforms!(u_unit_position: unit_render.position,),
        );
        // for layer in &unit_render.layers {
        //     self.draw_shader(framebuffer, layer);
        // }
    }
}
