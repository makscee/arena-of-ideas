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
    pub fn show(mut self, ctx: &egui::Context) -> T {
        TopBottomPanel::top("Subsection Menu")
            .frame(Frame::none().inner_margin(Margin {
                left: 13.0,
                top: 3.0,
                ..default()
            }))
            .resizable(false)
            .show_separator_line(false)
            .show(ctx, |ui| {
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
            });
        self.selected
    }
}
