use super::*;

pub struct UnitVisualEffect {
    pub id: Id,
}

impl UnitVisualEffect {
    pub fn new(id: Id) -> Self {
        Self { id }
    }
}

impl VisualEffect for UnitVisualEffect {
    fn draw(
        &self,
        render: &ViewRender,
        framebuffer: &mut ugli::Framebuffer,
        _t: Time,
        _model: &VisualNodeModel,
    ) {
        render.draw_shader(framebuffer, &render.assets.system_shaders.unit);
    }

    fn update(&self, model: &mut VisualNodeModel, t: Time) {
        #![allow(unused_variables)]
    }

    fn get_order(&self) -> i32 {
        1000
    }
}
