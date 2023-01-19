use super::*;

pub struct DrawModelUnitsVisualEffect {}

impl DrawModelUnitsVisualEffect {
    pub fn new() -> Self {
        Self {}
    }
}

impl VisualEffect for DrawModelUnitsVisualEffect {
    fn draw(
        &self,
        render: &ViewRender,
        framebuffer: &mut ugli::Framebuffer,
        _t: Time,
        model: &VisualNodeModel,
    ) {
        model
            .units
            .iter()
            .for_each(|unit| render.draw_unit(framebuffer, unit));
    }

    fn update(&self, model: &mut VisualNodeModel, t: Time) {
        #![allow(unused_variables)]
    }

    fn get_order(&self) -> i32 {
        1000
    }
}
