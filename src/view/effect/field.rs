use super::*;

pub struct FieldVisualEffect {
    pub shader: ShaderProgram,
}

impl FieldVisualEffect {
    pub fn new(shader: ShaderProgram) -> Self {
        Self { shader }
    }
}

impl VisualEffect for FieldVisualEffect {
    fn draw(&self, render: &ViewRender, framebuffer: &mut ugli::Framebuffer, t: Time) {
        render.draw_shader(framebuffer, &self.shader);
    }

    fn get_order(&self) -> i32 {
        -1000
    }
}
