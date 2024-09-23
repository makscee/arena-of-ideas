use std::ops::RangeInclusive;

use egui::emath::Numeric;

use super::*;

pub struct Slider {
    name: &'static str,
    show_name: bool,
    log: bool,
}

impl Slider {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            show_name: true,
            log: false,
        }
    }
    pub fn name(mut self, show: bool) -> Self {
        self.show_name = show;
        self
    }
    pub fn log(mut self) -> Self {
        self.log = true;
        self
    }
    pub fn ui<Num: Numeric>(
        self,
        value: &mut Num,
        range: RangeInclusive<Num>,
        ui: &mut Ui,
    ) -> Response {
        if self.show_name {
            ui.label(self.name);
        }
        ui.spacing_mut().slider_width = ui.available_width() - 80.0;
        let resp = egui::Slider::new(value, range)
            .handle_shape(HandleShape::Circle)
            .logarithmic(self.log)
            .trailing_fill(false)
            .smallest_positive(0.1)
            .ui(ui);
        ui.reset_style();
        resp
    }
}
