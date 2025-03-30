use super::*;

pub trait NodeView: NodeExt {
    fn view(&self, ui: &mut Ui, context: &Context) -> Result<(), ExpressionError> {
        let is_compact_id = ui.next_auto_id();
        let is_compact = get_ctx_bool_id_default(ui.ctx(), is_compact_id, true);
        if if is_compact {
            self.compact(ui, context)
        } else {
            self.full(ui, context)
        }?
        .clicked()
        {
            set_ctx_bool_id(ui.ctx(), is_compact_id, !is_compact);
        }
        Ok(())
    }
    fn compact(&self, ui: &mut Ui, context: &Context) -> Result<Response, ExpressionError> {
        let mut context = context.clone();
        let vars: HashMap<VarName, VarValue> = HashMap::from_iter(self.get_own_vars());
        for (var, value) in &vars {
            context.set_var(*var, value.clone());
        }
        let title = self.kind().to_string();
        let color = context
            .get_color(VarName::color)
            .unwrap_or(tokens_global().ui_element_border_and_focus_rings());
        Ok(show_frame(&title, color, ui, |ui| {
            for (var, value) in vars {
                ui.horizontal(|ui| {
                    var.cstr().label(ui);
                    value.cstr().label(ui);
                });
            }
        }))
    }
    fn full(&self, ui: &mut Ui, context: &Context) -> Result<Response, ExpressionError> {
        let mut context = context.clone();
        let vars: HashMap<VarName, VarValue> = HashMap::from_iter(self.get_own_vars());
        for (var, value) in &vars {
            context.set_var(*var, value.clone());
        }
        let title = self.kind().to_string();
        let color = context
            .get_color(VarName::color)
            .unwrap_or(tokens_global().ui_element_border_and_focus_rings());
        Ok(show_frame(&title, color, ui, |ui| {
            for (var, value) in vars {
                ui.horizontal(|ui| {
                    var.cstr().label(ui);
                    value.cstr().label(ui);
                });
            }
        }))
    }
}

fn show_frame(title: &str, color: Color32, ui: &mut Ui, content: impl FnOnce(&mut Ui)) -> Response {
    Frame {
        inner_margin: Margin::ZERO,
        outer_margin: MARGIN,
        fill: ui.visuals().faint_bg_color,
        stroke: color.stroke(),
        corner_radius: ROUNDING,
        shadow: Shadow::NONE,
    }
    .show(ui, |ui| {
        const R: u8 = ROUNDING.ne;
        const M: i8 = 6;
        let response = Frame::new()
            .corner_radius(CornerRadius {
                nw: R,
                ne: 0,
                sw: 0,
                se: R,
            })
            .fill(color)
            .inner_margin(Margin {
                left: M,
                right: M,
                top: 0,
                bottom: 0,
            })
            .show(ui, |ui| {
                title
                    .cstr_cs(ui.visuals().faint_bg_color, CstrStyle::Bold)
                    .as_label(ui.style())
                    .sense(Sense::click())
                    .ui(ui)
            })
            .inner;
        Frame::new()
            .inner_margin(Margin {
                left: M,
                right: M,
                top: 0,
                bottom: M,
            })
            .show(ui, content);
        response
    })
    .inner
}

impl NodeView for All {}
impl NodeView for Incubator {}
impl NodeView for Player {}
impl NodeView for PlayerData {}
impl NodeView for PlayerIdentity {}
impl NodeView for House {
    fn compact(&self, ui: &mut Ui, context: &Context) -> Result<Response, ExpressionError> {
        let name = self
            .get_var(VarName::name, &context)
            .to_e("Name not found")?
            .get_string()?;
        let color = self
            .get_var(VarName::color, context)
            .to_e("Failed to get color")?
            .get_color()?;
        let context = &context.clone().set_var(VarName::color, color.into()).take();
        Ok(show_frame(&name, color, ui, |ui| {
            ui.horizontal(|ui| {
                "ability:".cstr_c(ui.visuals().weak_text_color()).label(ui);
                if let Some(ability) = self.action_ability_load(context) {
                    ability.view(ui, context).ui(ui);
                }
            });
            ui.horizontal(|ui| {
                "units:".cstr_c(ui.visuals().weak_text_color()).label(ui);
                ui.vertical(|ui| {
                    for unit in self.units_load(context) {
                        unit.view(ui, context).ui(ui);
                    }
                })
            });
        }))
    }
}
impl NodeView for HouseColor {}
impl NodeView for ActionAbility {
    fn compact(&self, ui: &mut Ui, context: &Context) -> Result<Response, ExpressionError> {
        let name = self
            .get_var(VarName::name, context)
            .to_e("Failed to get unit name")?
            .get_string()?;
        let color = context.get_color(VarName::color)?;
        Ok(TagWidget::new_name(name, color).ui(ui))
    }
}
impl NodeView for ActionAbilityDescription {}
impl NodeView for AbilityEffect {}
impl NodeView for StatusAbility {}
impl NodeView for StatusAbilityDescription {}
impl NodeView for Team {}
impl NodeView for Match {}
impl NodeView for ShopCaseUnit {}
impl NodeView for Fusion {}
impl NodeView for Unit {
    fn compact(&self, ui: &mut Ui, context: &Context) -> Result<Response, ExpressionError> {
        let name = self
            .get_var(VarName::name, context)
            .to_e("Failed to get unit name")?
            .get_string()?;
        let color = context.get_color(VarName::color)?;
        let pwr = self
            .get_var(VarName::pwr, context)
            .to_e("Failed to get pwr")?
            .cstr_c(VarName::pwr.color());
        let hp = self
            .get_var(VarName::hp, context)
            .to_e("Failed to get hp")?
            .cstr_c(VarName::hp.color());
        let stats = format!("{}/{}", pwr, hp);
        Ok(TagWidget::new_name_value(name, color, stats).ui(ui))
    }
    fn full(&self, ui: &mut Ui, context: &Context) -> Result<Response, ExpressionError> {
        Ok(UnitCard {
            name: self.name.clone(),
            description: String::new(),
            house: String::new(),
            house_color: Color32::default(),
            rarity: Rarity::default(),
            behavior: Behavior::default(),
            vars: HashMap::new(),
            expanded: false,
        }
        .show(context, ui))
    }
}
impl NodeView for UnitDescription {}
impl NodeView for UnitStats {}
impl NodeView for Behavior {}
impl NodeView for Representation {}
