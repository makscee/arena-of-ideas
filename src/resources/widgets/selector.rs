use egui::ComboBox;

use super::*;

pub struct Selector {
    name: &'static str,
}

impl Selector {
    pub fn new(name: &'static str) -> Self {
        Self { name }
    }
    pub fn ui_enum<E: ToCstr + IntoEnumIterator + Copy + PartialEq>(
        self,
        value: &mut E,
        ui: &mut Ui,
    ) {
        ComboBox::from_label(self.name.cstr().widget(1.0, ui))
            .selected_text(value.cstr().widget(1.0, ui))
            .show_ui(ui, |ui| {
                for e in E::iter() {
                    let text = e.cstr().widget(1.0, ui);
                    ui.selectable_value(value, e.clone(), text);
                }
            });
    }
    pub fn ui_iter<'a, E: PartialEq + Clone + ToString + ToCstr + 'a, I>(
        self,
        value: &mut E,
        values: I,
        ui: &mut Ui,
    ) where
        I: IntoIterator<Item = &'a E>,
    {
        self.name.cstr().label(ui);
        ComboBox::from_id_source(self.name)
            .selected_text(value.cstr_c(name_color(&value.to_string())).widget(1.0, ui))
            .show_ui(ui, |ui| {
                for e in values {
                    let text = e.cstr_c(name_color(&e.to_string())).widget(1.0, ui);
                    ui.selectable_value(value, e.clone(), text);
                }
            });
    }
}
