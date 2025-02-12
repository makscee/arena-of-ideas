use super::*;

#[derive(Default)]
pub struct UnitCard {
    pub name: String,
    pub description: String,
    pub house: String,
    pub house_color: Color32,
    pub rarity: Rarity,
    pub reaction: Reaction,
    pub vars: HashMap<VarName, VarValue>,
    pub expanded: bool,
}

impl UnitCard {
    pub fn show(&self, ui: &mut Ui) {
        ui.spacing_mut().item_spacing.y = 1.0;
        Frame::none()
            .fill(BG_DARK)
            .stroke(Stroke::new(2.0, self.rarity.color()))
            .rounding(Rounding {
                nw: 13.0,
                ne: 13.0,
                sw: 0.0,
                se: 0.0,
            })
            .show(ui, |ui| {
                ui.vertical_centered_justified(|ui| {
                    self.name
                        .cstr_cs(self.house_color, CstrStyle::Heading)
                        .label(ui);
                    self.show_tags(ui);
                });
            });
        Frame::none()
            .fill(EMPTINESS)
            .inner_margin(Margin::same(4.0))
            .rounding(Rounding {
                nw: 0.0,
                ne: 0.0,
                sw: 13.0,
                se: 13.0,
            })
            .stroke(STROKE_DARK)
            .show(ui, |ui| {
                if self.expanded {
                    ui.vertical(|ui| {
                        self.show_description(ui);
                        ui.vertical(|ui| {
                            for (trigger, actions) in &self.reaction.trigger {
                                trigger.cstr().label(ui);
                                for a in actions.0.iter() {
                                    let r = a.cstr().label_w(ui);
                                    let rect = r
                                        .rect
                                        .translate(egui::vec2(-3.0, 0.0))
                                        .shrink2(egui::vec2(0.0, 3.0));
                                    ui.painter().line_segment(
                                        [rect.left_top(), rect.left_bottom()],
                                        STROKE_YELLOW,
                                    );
                                }
                            }
                        });
                    });
                } else {
                    self.show_description(ui);
                }
                ui.expand_to_include_rect(ui.available_rect_before_wrap());
            });
    }
    fn show_tags(&self, ui: &mut Ui) {
        if let Some(hp) = self.vars.get(&VarName::hp).and_then(|v| v.get_i32().ok()) {
            if let Some(pwr) = self.vars.get(&VarName::pwr).and_then(|v| v.get_i32().ok()) {
                let dmg = self
                    .vars
                    .get(&VarName::dmg)
                    .and_then(|v| v.get_i32().ok())
                    .unwrap_or_default();
                let mut tags = TagsWidget::new();
                tags.add_number("pwr", YELLOW, pwr);
                tags.add_number_cstr(
                    "hp",
                    RED,
                    format!(
                        "[b {}[vd /]{}]",
                        (hp - dmg).to_string().cstr_c(VISIBLE_BRIGHT),
                        hp.to_string().cstr_c(RED)
                    ),
                );
                tags.ui(ui);
            }
        }
        let mut tags = TagsWidget::new();
        tags.add_text(self.house.to_owned(), self.house_color);
        tags.add_text(self.rarity.to_string(), self.rarity.color());
        if let Some(lvl) = self.vars.get(&VarName::lvl).and_then(|v| v.get_i32().ok()) {
            tags.add_number("lvl", PURPLE, lvl);
        }
        if let Some(tier) = self.vars.get(&VarName::tier).and_then(|v| v.get_i32().ok()) {
            tags.add_number("tier", YELLOW, tier);
        }
        tags.ui(ui);
    }
    fn show_description(&self, ui: &mut Ui) {
        Frame::none()
            .fill(if self.expanded { BG_DARK } else { TRANSPARENT })
            .inner_margin(Margin::same(6.0))
            .rounding(Rounding::same(13.0))
            .show(ui, |ui| {
                ui.vertical_centered_justified(|ui| {
                    self.description.cstr().label_w(ui);
                });
            });
    }
}
