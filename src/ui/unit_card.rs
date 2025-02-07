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
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            for (var, value) in self.vars.iter().sorted_by_key(|(v, _)| **v as u8) {
                                format!(
                                    "[vd {}:] {}",
                                    var,
                                    value.cstr_cs(var.color(), CstrStyle::Bold)
                                )
                                .label(ui);
                            }
                        });
                        ui.vertical(|ui| {
                            self.show_description(ui);
                            ui.vertical_centered_justified(|ui| {
                                self.reaction.trigger.cstr().label(ui);
                            });
                            ui.vertical(|ui| {
                                for a in &self.reaction.actions.0 {
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
                            });
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
                format!(
                    "[b {}[vd /]{}]",
                    pwr.cstr_c(VarName::pwr.color()),
                    hp.cstr_c(VarName::hp.color()),
                )
                .label(ui);
            }
        }
        let mut tags = [
            (self.house.to_owned(), self.house_color),
            (self.rarity.to_string(), self.rarity.color()),
        ]
        .to_vec();
        if let Some(lvl) = self.vars.get(&VarName::lvl).and_then(|v| v.get_i32().ok()) {
            tags.push((format!("lvl {lvl}"), PURPLE));
        }
        if let Some(tier) = self.vars.get(&VarName::tier).and_then(|v| v.get_i32().ok()) {
            tags.push((format!("tier {tier}"), YELLOW));
        }
        TagsWidget::new(tags).ui(ui);
    }
    fn show_description(&self, ui: &mut Ui) {
        Frame::none()
            .fill(BG_DARK)
            .inner_margin(Margin::same(6.0))
            .rounding(Rounding::same(13.0))
            .show(ui, |ui| {
                ui.vertical_centered_justified(|ui| {
                    self.description.cstr().label_w(ui);
                });
            });
    }
}
