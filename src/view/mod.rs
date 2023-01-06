use super::*;

mod effect;
mod node;
mod queue;
mod shader;

pub use effect::*;
pub use geng::Camera2d;
pub use node::*;
pub use queue::*;
pub use shader::*;

pub struct View {
    pub queue: VisualQueue,
    pub camera: Camera2d,
    geng: Geng,
    assets: Rc<Assets>,
}

impl View {
    pub fn new(geng: Geng, assets: Rc<Assets>) -> Self {
        let queue = VisualQueue {
            nodes: VecDeque::new(),
            persistent_nodes: vec![],
        };
        let camera = geng::Camera2d {
            center: vec2(0.0, 0.0),
            rotation: 0.0,
            fov: 5.0,
        };
        Self {
            queue,
            camera,
            geng,
            assets,
        }
    }

    pub fn draw(&self, framebuffer: &mut ugli::Framebuffer) {
        self.draw_field(framebuffer);
    }

    fn draw_field(&self, framebuffer: &mut ugli::Framebuffer) {
        let shader_program = &self.assets.system_shaders.field;
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
}
