use super::*;

mod field;
mod unit;

pub use field::*;
pub use unit::*;

pub trait VisualEffect {
    fn draw(
        &self,
        render: &ViewRender,
        framebuffer: &mut ugli::Framebuffer,
        t: Time,
        model: &VisualNodeModel,
    ) {
        #![allow(unused_variables)]
    }
    fn update(&self, model: &mut VisualNodeModel, t: Time) {
        #![allow(unused_variables)]
    }
    fn get_order(&self) -> i32 {
        0
    }
}
