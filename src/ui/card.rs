use super::*;

#[derive(Clone, Copy, Default)]
struct TagCardState {
    expanded: bool,
}

impl TagCardState {
    fn expanded(mut self, value: bool) -> Self {
        self.expanded = value;
        self
    }
    fn get(node: &impl Node, ui: &mut Ui) -> Self {
        ui.ctx()
            .data(|r| r.get_temp::<TagCardState>(node.egui_id()))
            .unwrap_or_default()
    }
    fn save(self, node: &impl Node, ui: &mut Ui) {
        ui.ctx().data_mut(|w| w.insert_temp(node.egui_id(), self));
    }
}

impl Unit {
    pub fn tag_card(&self, context: &Context, ui: &mut Ui) -> Result<(), ExpressionError> {
        let state = TagCardState::get(self, ui);
        let response = if state.expanded {
            self.show_card(context, ui)?
        } else {
            self.show_tag(context, ui)?
        };
        if response.clicked() {
            state.expanded(!state.expanded).save(self, ui);
        }
        Ok(())
    }
    pub fn show_tag(&self, context: &Context, ui: &mut Ui) -> Result<Response, ExpressionError> {
        Ok(TagWidget::new_name_value(
            context.get_string(VarName::unit_name)?,
            context.get_color(VarName::color)?,
            format!(
                "[b {} {}]",
                context.get_i32(VarName::pwr)?.cstr_c(VarName::pwr.color()),
                context.get_i32(VarName::hp)?.cstr_c(VarName::hp.color())
            ),
        )
        .ui(ui))
    }
    pub fn show_card(&self, context: &Context, ui: &mut Ui) -> Result<Response, ExpressionError> {
        let color = context.get_color(VarName::color)?;
        let pwr = context.get_var(VarName::pwr)?;
        let hp = context.get_var(VarName::hp)?;
        Ok(Frame::new()
            .inner_margin(2)
            .corner_radius(ROUNDING)
            .stroke(color.stroke())
            .show(ui, |ui| {
                let response = ui
                    .horizontal(|ui| {
                        let response = self.unit_name.clone().as_button().color(color, ui).ui(ui);
                        RectButton::new_size(8.0.v2())
                            .ui(ui, |color, rect, _, ui| {
                                ui.painter().line_segment(
                                    [rect.center_top(), rect.center_bottom()],
                                    color.stroke(),
                                );
                                ui.painter().line_segment(
                                    [rect.left_center(), rect.right_center()],
                                    color.stroke(),
                                );
                            })
                            .bar_menu(|ui| {
                                ui.menu_button("inspect", |ui| {
                                    self.view(ViewContext::new(ui), context, ui);
                                });
                            });
                        response
                    })
                    .inner;
                ui.horizontal(|ui| {
                    TagWidget::new_var_value(VarName::pwr, pwr).ui(ui);
                    TagWidget::new_var_value(VarName::hp, hp).ui(ui);
                });
                if let Some(description) = self.description_load(context) {
                    description.description.label_w(ui);
                }
                response
            })
            .inner)
    }
}
impl House {
    pub fn show_card(&self, context: &Context, ui: &mut Ui) -> Result<(), ExpressionError> {
        let color = self
            .color_load(context)
            .and_then(|c| c.color.try_c32().ok())
            .unwrap_or_else(|| ui.visuals().text_color());
        Frame::new()
            .inner_margin(2)
            .corner_radius(ROUNDING)
            .stroke(color.stroke())
            .show(ui, |ui| {
                let resp = self.house_name.cstr_c(color).button(ui);
                resp.bar_menu(|ui| {
                    ui.menu_button("inspect", |ui| {
                        ui.reset_style();
                        self.view(ViewContext::new(ui), context, ui)
                    });
                });
                for unit in self.units_load(context) {
                    unit.tag_card(context.clone().set_owner(unit.entity()), ui)
                        .ui(ui);
                }
            });
        Ok(())
    }
}
impl Fusion {
    pub fn show_card(&self, context: &Context, ui: &mut Ui) -> Result<(), ExpressionError> {
        let units = self.units(context)?;
        let mut pwr = 0;
        let mut hp = 0;
        for unit in &units {
            let stats = unit.description_err(context)?.stats_err(context)?;
            pwr += stats.pwr;
            hp += stats.hp;
        }
        Frame::new()
            .fill(ui.visuals().window_fill)
            .corner_radius(ROUNDING)
            .inner_margin(MARGIN)
            .show(ui, |ui| -> Result<(), ExpressionError> {
                ui.horizontal_wrapped(|ui| {
                    TagWidget::new_var_value(VarName::pwr, pwr.into()).ui(ui);
                    TagWidget::new_var_value(VarName::hp, hp.into()).ui(ui);
                });
                ui.vertical(|ui| {
                    for unit in &units {
                        unit.tag_card(context.clone().set_owner(unit.entity()), ui)
                            .ui(ui);
                    }
                });
                for (tr, actions) in &self.behavior {
                    if actions.is_empty() {
                        continue;
                    }
                    let trigger = self.get_trigger(tr, context)?;
                    let view_ctx = ViewContext::new(ui).non_interactible(true);
                    ui.horizontal(|ui| {
                        Icon::Lightning.show(ui);
                        trigger.show_title(view_ctx, context, ui);
                    });
                    for ar in actions {
                        let (entity, action) = self.get_action(ar, context)?;
                        let action = action.clone();
                        action.show_title(view_ctx, context.clone().set_owner(entity), ui);
                    }
                }
                Ok(())
            })
            .inner?;
        Ok(())
    }
}
