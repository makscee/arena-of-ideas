use super::*;

pub trait VisualEffect {
    fn draw(&self, framebuffer: &mut ugli::Framebuffer, t: Time);
    fn update(&self, model: &mut VisualNodeModel, t: Time);
    fn get_order(&self) -> i32;
}
