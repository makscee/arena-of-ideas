use std::rc::Rc;

use super::*;

mod effect;
mod node;
mod queue;
mod render;
mod shader;
mod unit_render;

pub use effect::*;
pub use geng::Camera2d;
pub use node::*;
pub use queue::*;
pub use render::*;
pub use shader::*;
pub use unit_render::*;

pub struct View {
    pub queue: VisualQueue,
    pub render: ViewRender,
}

impl View {
    pub fn new(geng: Geng, assets: Rc<Assets>) -> Self {
        let mut queue = VisualQueue::new();
        let camera = geng::Camera2d {
            center: vec2(0.0, 0.0),
            rotation: 0.0,
            fov: 5.0,
        };
        queue
            .persistent_effects
            .push(Box::new(FieldVisualEffect::new(
                assets.system_shaders.field.clone(),
            )));
        let render = ViewRender::new(camera, geng, assets);
        Self { queue, render }
    }

    pub fn add_unit_to_render(&mut self, unit: Unit) {
        todo!();
    }

    pub fn draw(&self, framebuffer: &mut ugli::Framebuffer, game_time: Time) {
        self.queue.draw(&self.render, framebuffer, game_time);
    }

    pub fn update(&mut self, timestamp: Time) {
        self.queue.update(timestamp);
    }
}
