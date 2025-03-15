use super::*;

#[derive(Default)]
pub struct UnitCard {
    pub name: String,
    pub description: String,
    pub house: String,
    pub house_color: Color32,
    pub rarity: Rarity,
    pub behavior: Behavior,
    pub vars: HashMap<VarName, VarValue>,
    pub expanded: bool,
}

const VARS: &'static [VarName] = &[
    VarName::hp,
    VarName::pwr,
    VarName::dmg,
    VarName::lvl,
    VarName::tier,
    VarName::color,
];

impl UnitCard {
    pub fn from_context(context: &Context) -> Result<Self, ExpressionError> {
        let vars = context.get_vars(VARS.iter().copied());
        Ok(Self {
            name: context.get_string(VarName::name)?,
            description: context.get_string(VarName::description)?,
            house: context
                .find_parent_component::<House>(context.get_owner()?)
                .to_e("House not found")?
                .name
                .clone(),
            house_color: context
                .find_parent_component::<HouseColor>(context.get_owner()?)
                .to_e("HouseColor not found")?
                .color
                .c32(),
            rarity: Rarity::default(),
            behavior: Behavior::default(),
            vars,
            expanded: false,
        })
    }
    pub fn show(&self, ui: &mut Ui) {
        ui.spacing_mut().item_spacing.y = 1.0;
        Frame::new()
            .fill(BG_DARK)
            .stroke(Stroke::new(2.0, self.rarity.color()))
            .corner_radius(CornerRadius {
                nw: 13,
                ne: 13,
                sw: 0,
                se: 0,
            })
            .show(ui, |ui| {
                ui.vertical_centered_justified(|ui| {
                    self.name
                        .cstr_cs(self.house_color, CstrStyle::Heading)
                        .label(ui);
                    self.show_tags(ui);
                });
            });
        Frame::new()
            .fill(EMPTINESS)
            .inner_margin(Margin::same(4))
            .corner_radius(CornerRadius {
                nw: 0,
                ne: 0,
                sw: 13,
                se: 13,
            })
            .stroke(STROKE_BG_DARK)
            .show(ui, |ui| {
                if self.expanded {
                    ui.vertical(|ui| {
                        self.show_description(ui);
                        ui.vertical(|ui| {
                            for Reaction { trigger, actions } in &self.behavior.triggers {
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
        Frame::new()
            .fill(if self.expanded { BG_DARK } else { TRANSPARENT })
            .inner_margin(Margin::same(6))
            .corner_radius(CornerRadius::same(13))
            .show(ui, |ui| {
                ui.vertical_centered_justified(|ui| {
                    self.description.cstr().label_w(ui);
                });
            });
    }
}

pub fn show_unit_tag(unit: &Unit, stats: &UnitStats, ui: &mut Ui, world: &World) {
    TagWidget::new_number(
        &unit.name,
        Context::new_world(world)
            .set_owner(unit.entity())
            .get_color(VarName::color)
            .unwrap(),
        format!(
            "[b {} {}]",
            stats.pwr.cstr_c(VarName::pwr.color()),
            stats.hp.cstr_c(VarName::hp.color())
        ),
    )
    .ui(ui);
}
