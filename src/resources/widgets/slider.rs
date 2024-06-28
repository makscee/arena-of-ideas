use std::ops::RangeInclusive;

use egui::DragValue;

use super::*;

pub struct Slider {
    name: &'static str,
    range: RangeInclusive<f32>,
    show_name: bool,
    log: bool,
}

impl Slider {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            show_name: true,
            log: false,
            range: 0.0..=1.0,
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
    pub fn range(mut self, range: RangeInclusive<f32>) -> Self {
        self.range = range;
        self
    }
    pub fn ui(self, value: &mut f32, ui: &mut Ui) -> Response {
        if self.show_name {
            ui.label(self.name);
        }
        ui.spacing_mut().slider_width = ui.available_width() - 80.0;
        let resp = egui::Slider::new(value, self.range.clone())
            .handle_shape(HandleShape::Circle)
            .logarithmic(self.log)
            .max_decimals(2)
            .trailing_fill(false)
            .smallest_positive(0.1)
            .ui(ui);
        ui.reset_style();
        resp
    }
}
