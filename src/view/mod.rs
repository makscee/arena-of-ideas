use super::*;

mod effect;
mod node;
mod queue;
mod shader;
mod unit_render;

pub use effect::*;
pub use geng::Camera2d;
pub use node::*;
pub use queue::*;
pub use shader::*;
pub use unit_render::*;

pub struct View {
    units: Collection<UnitRender>,
    pub queue: VisualQueue,
    camera: Camera2d,
    geng: Geng,
    assets: Rc<Assets>,
}

impl View {
    pub fn new(geng: Geng, assets: Rc<Assets>) -> Self {
        let queue = VisualQueue::new();
        let camera = geng::Camera2d {
            center: vec2(0.0, 0.0),
            rotation: 0.0,
            fov: 5.0,
        };
        Self {
            units: default(),
            queue,
            camera,
            geng,
            assets,
        }
    }

    pub fn add_unit_to_render(&mut self, unit: Unit) {
        self.units.insert(UnitRender::new_from_unit(unit));
    }

    pub fn draw(&self, framebuffer: &mut ugli::Framebuffer) {
        self.draw_field(framebuffer);
        self.units
            .iter()
            .for_each(|u| self.draw_unit(framebuffer, u));
    }

    pub fn draw_shader(&self, framebuffer: &mut ugli::Framebuffer, shader_program: &ShaderProgram) {
        let mut instances_arr: ugli::VertexBuffer<Instance> =
            ugli::VertexBuffer::new_dynamic(self.geng.ugli(), Vec::new());
        instances_arr.resize(shader_program.instances, Instance {});
        let quad = shader_program.get_vertices(&self.geng);
        let framebuffer_size = framebuffer.size();

        let program = shader_program
            .program
            .as_ref()
            .expect("Shader program not loaded");
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
            ),
            ugli::DrawParameters {
                blend_mode: Some(ugli::BlendMode::default()),
                ..default()
            },
        );
    }

    fn draw_field(&self, framebuffer: &mut ugli::Framebuffer) {
        self.draw_shader(framebuffer, &self.assets.system_shaders.field);
    }

    fn draw_unit(&self, framebuffer: &mut ugli::Framebuffer, unit_render: &UnitRender) {
        self.draw_shader(framebuffer, &self.assets.system_shaders.unit);
        for layer in &unit_render.layers {
            self.draw_shader(framebuffer, layer);
        }
    }
}
