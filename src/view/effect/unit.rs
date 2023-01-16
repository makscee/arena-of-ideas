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
        model: &VisualNodeModel,
    ) {
        let unit = model.units.get(&self.id);
        if let Some(unit) = unit {
            debug!("unit position: {}", unit.position);
            render.draw_unit(framebuffer, unit);
        }
    }

    fn update(&self, model: &mut VisualNodeModel, t: Time) {
        #![allow(unused_variables)]
    }

    fn get_order(&self) -> i32 {
        1000
    }
}
