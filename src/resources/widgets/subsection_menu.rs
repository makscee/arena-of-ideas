use super::*;

pub struct SubsectionMenu<T> {
    selected: T,
}

impl<T> SubsectionMenu<T>
where
    T: IntoEnumIterator + ToString + PartialEq + Copy,
{
    pub fn new(selected: T) -> Self {
        Self { selected }
    }
    pub fn show(mut self, ui: &mut Ui) -> T {
        ui.horizontal(|ui| {
            ui.visuals_mut().widgets.hovered.bg_fill = VISIBLE_LIGHT;
            for i in T::iter() {
                let active = i.eq(&self.selected);
                let name = i.to_string();
                if Button::click(name).active(active).ui(ui).clicked() {
                    self.selected = i;
                }
            }
        });
        self.selected
    }
}
