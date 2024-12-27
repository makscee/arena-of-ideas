use super::*;

pub struct Slider {
    name: &'static str,
    show_name: bool,
    log: bool,
    full_width: bool,
}

impl Slider {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            show_name: true,
            log: false,
            full_width: false,
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
    pub fn full_width(mut self) -> Self {
        self.full_width = true;
        self
    }
    pub fn ui<Num: Numeric>(
        self,
        value: &mut Num,
        range: RangeInclusive<Num>,
        ui: &mut Ui,
    ) -> bool {
        if self.show_name {
            ui.label(self.name);
        }
        if self.full_width {
            let width = ui.available_width() - 80.0;
            if width < 5.0 {
                return false;
            }
            ui.spacing_mut().slider_width = width;
        }
        let changed = egui::Slider::new(value, range)
            .handle_shape(HandleShape::Circle)
            .logarithmic(self.log)
            .trailing_fill(true)
            .smallest_positive(0.1)
            .ui(ui)
            .changed();
        ui.reset_style();
        changed
    }
}
