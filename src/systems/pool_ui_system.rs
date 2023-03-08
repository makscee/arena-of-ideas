use geng::ui::*;

use super::*;

pub struct PoolUiSystem {
    need_open_pool: bool,
    field_height: f32,
}

impl PoolUiSystem {
    pub fn new() -> Self {
        Self {
            need_open_pool: false,
            field_height: 0.0,
        }
    }
}

impl System for PoolUiSystem {
    fn ui<'a>(
        &'a mut self,
        cx: &'a ui::Controller,
        world: &'a legion::World,
        resources: &'a Resources,
    ) -> Box<dyn ui::Widget + 'a> {
        let pool_button =
            CornerButtonWidget::new(cx, resources, resources.options.images.pool_icon.clone());
        if pool_button.was_clicked() {
            self.need_open_pool = !self.need_open_pool;
        }
        let pool_widget = PoolWidget::new(cx, resources, self.field_height);
        Box::new((pool_button.place(vec2(0.0, 1.0)), pool_widget.place()).stack())
    }

    fn update(&mut self, world: &mut legion::World, resources: &mut Resources) {
        let need_height = if self.need_open_pool { 1.0 } else { 0.0 };
        self.field_height += (need_height - self.field_height) * resources.delta_time * 5.0;
        // if self.need_open_pool
        //     && resources
        //         .down_mouse_buttons
        //         .contains(&geng::MouseButton::Left)
        // {
        //     self.need_open_pool = false;
        // }
    }
}
