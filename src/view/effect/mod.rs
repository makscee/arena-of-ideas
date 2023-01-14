use super::*;

mod field;

pub use field::*;

pub trait VisualEffect {
    fn draw(&self, render: &ViewRender, framebuffer: &mut ugli::Framebuffer, t: Time) {
        #![allow(unused_variables)]
    }
    fn update(&self, model: &mut VisualNodeModel, t: Time) {
        #![allow(unused_variables)]
    }
    fn get_order(&self) -> i32 {
        0
    }
}
