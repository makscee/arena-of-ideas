use super::*;

#[derive(Default)]
pub struct UnitCard {
    pub name: String,
    pub description: String,
    pub house: String,
    pub house_color: Color32,
    pub rarity: Rarity,
    pub vars: HashMap<VarName, VarValue>,
}

impl UnitCard {
    pub fn show(&self, ui: &mut Ui) {
        let width = ui.available_width();
        ui.vertical_centered_justified(|ui| {
            self.name
                .cstr_cs(self.house_color, CstrStyle::Heading2)
                .label(ui);
            let tag = TagWidget::new(&self.house, self.house_color);
            let mut tags = [tag.clone(), tag.clone(), tag.clone()];
            let mut size = egui::Vec2::ZERO;
            for tag in &mut tags {
                let tag_size = tag.size(ui);
                size.y = size.y.max(tag_size.y);
                size.x += tag_size.x;
            }
            let right_bottom = ui.cursor().center_top() + egui::vec2(size.x * 0.5, size.y);
            let rect = Rect::from_min_max(right_bottom - size, right_bottom);
            ui.allocate_ui_at_rect(rect, |ui| {
                ui.horizontal(|ui| {
                    for tag in tags {
                        tag.ui(ui);
                    }
                })
            });
        });
    }
}
