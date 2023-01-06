use super::*;

mod effect;
mod node;
mod queue;

pub use effect::*;
pub use geng::Camera2d;
pub use node::*;
pub use queue::*;

pub struct View {
    pub queue: VisualQueue,
    pub camera: Camera2d,
}

impl View {
    pub fn draw(&self, framebuffer: &mut ugli::Framebuffer) {}
}
