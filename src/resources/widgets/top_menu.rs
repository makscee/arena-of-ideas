use super::*;

pub struct TopMenu {
    buttons: Vec<TopMenuButton>,
}

struct TopMenuButton {
    name: &'static str,
}

impl TopMenu {
    pub fn new(btns: Vec<&'static str>) -> Self {
        Self {
            buttons: btns
                .into_iter()
                .map(|name| TopMenuButton { name })
                .collect_vec(),
        }
    }
    pub fn ui(self, ui: &mut Ui) {
        TopBottomPanel::top("Top Menu")
            .frame(Frame::none().inner_margin(Margin::same(13.0)))
            .resizable(false)
            .show_separator_line(false)
            .show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.visuals_mut().widgets.hovered.bg_fill = LIGHT_GRAY;
                    for TopMenuButton { name } in self.buttons {
                        let path = &format!("/{name}");
                        let enabled = ui.ctx().is_path_enabled(path);
                        ui.visuals_mut().widgets.inactive.fg_stroke.color =
                            if enabled { LIGHT_GRAY } else { GRAY };
                        let resp = egui::Button::new(name)
                            .min_size(egui::vec2(75.0, 0.0))
                            .ui(ui);
                        if resp.clicked() {
                            ui.ctx().flip_path_enabled(path);
                        }
                        let line_offset = egui::vec2(5.0, 0.0);
                        ui.painter().line_segment(
                            [
                                resp.rect.right_top() + line_offset,
                                resp.rect.right_bottom() + line_offset,
                            ],
                            ui.visuals().widgets.inactive.fg_stroke,
                        );
                    }
                });
            });
    }
}
