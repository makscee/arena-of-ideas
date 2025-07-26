use super::*;

pub trait SFnCard {
    fn see_card(&self, context: &Context, ui: &mut Ui) -> Result<Response, ExpressionError>;
}

impl SFnCard for NUnit {
    fn see_card(&self, context: &Context, ui: &mut Ui) -> Result<Response, ExpressionError> {
        let color = context.color(ui);
        let pwr = context.get_var(VarName::pwr)?;
        let hp = context.get_var(VarName::hp)?;
        let tier = if let Ok(behavior) = context.first_parent_recursive::<NUnitBehavior>(self.id) {
            behavior.reactions.tier()
        } else {
            0
        };
        Ok(show_frame(
            self,
            &self.unit_name,
            color,
            context,
            ui,
            |ui| {
                ui.horizontal(|ui| {
                    TagWidget::new_var_value(VarName::pwr, pwr).ui(ui);
                    TagWidget::new_var_value(VarName::hp, hp).ui(ui);
                    TagWidget::new_var_value(VarName::tier, (tier as i32).into()).ui(ui);
                });
                if let Ok(description) = self.description_load(context) {
                    description.description.label_w(ui);
                    if let Ok(behavior) = description.behavior_load(context) {
                        behavior.show(context, ui);
                    }
                }
            },
        ))
    }
}

impl SFnCard for NHouse {
    fn see_card(&self, context: &Context, ui: &mut Ui) -> Result<Response, ExpressionError> {
        let color = context.color(ui);
        Ok(show_frame(
            self,
            &self.house_name,
            color,
            context,
            ui,
            |ui| {
                if let Ok(action) = self.action_load(context) {
                    action.see(context).tag_card(ui).ok();
                }
                if let Ok(status) = self.status_load(context) {
                    status.see(context).tag_card(ui).ok();
                }
                for unit in self.units_load(context) {
                    context
                        .with_owner_ref(unit.entity(), |context| {
                            unit.clone().see(context).tag_card(ui)
                        })
                        .ok();
                }
            },
        ))
    }
}

impl SFnCard for NActionAbility {
    fn see_card(&self, context: &Context, ui: &mut Ui) -> Result<Response, ExpressionError> {
        let color = context.color(ui);
        Ok(show_frame(
            self,
            &self.ability_name,
            color,
            context,
            ui,
            |ui| {
                if let Ok(description) = self.description_load(context) {
                    description.description.label_w(ui);
                    if let Ok(effect) = description.effect_load(context) {
                        effect.show(context, ui);
                    }
                }
            },
        ))
    }
}

impl SFnCard for NStatusAbility {
    fn see_card(&self, context: &Context, ui: &mut Ui) -> Result<Response, ExpressionError> {
        let color = context.color(ui);
        Ok(show_frame(
            self,
            &self.status_name,
            color,
            context,
            ui,
            |ui| {
                if let Ok(description) = self.description_load(context) {
                    description.description.label_w(ui);
                    if let Ok(behavior) = description.behavior_load(context) {
                        behavior.show(context, ui);
                    }
                }
            },
        ))
    }
}

fn show_frame(
    _node: &impl ViewFns,
    name: &str,
    color: Color32,
    _context: &Context,
    ui: &mut Ui,
    content: impl FnOnce(&mut Ui),
) -> Response {
    Frame::new()
        .inner_margin(2)
        .corner_radius(ROUNDING)
        .stroke(color.stroke())
        .show(ui, |ui| {
            let resp = ui
                .horizontal(|ui| {
                    let resp = name.cstr_c(color).button(ui);
                    RectButton::new_size(5.0.v2())
                        .ui(ui, |color, rect, _, ui| {
                            ui.painter()
                                .circle_filled(rect.center(), rect.width() * 0.5, color);
                        })
                        .bar_menu(|ui| {
                            ui.menu_button("inspect", |ui| {
                                ui.label("Inspection removed - use new see() API");
                            });
                        });
                    resp
                })
                .inner;
            content(ui);
            resp
        })
        .inner
}

impl NFusion {
    pub fn show_card(&self, context: &Context, ui: &mut Ui) -> Result<(), ExpressionError> {
        ui.horizontal(|ui| {
            self.entity()
                .to_string()
                .cstr_cs(ui.visuals().weak_text_color(), CstrStyle::Small)
                .label(ui);
        });
        let units = self.units(context)?;
        context.with_owner_ref(self.entity(), |context| {
            let pwr = context.sum_var(VarName::pwr)?;
            let hp = context.sum_var(VarName::hp)?;
            let lvl = context.get_var(VarName::lvl)?;
            ui.horizontal(|ui| {
                TagWidget::new_var_value(VarName::pwr, pwr).ui(ui);
                TagWidget::new_var_value(VarName::hp, hp).ui(ui);
                TagWidget::new_var_value(VarName::lvl, lvl).ui(ui);
                ui.label(format!(
                    "Actions: {}/{}",
                    self.get_action_count(),
                    self.action_limit
                ));
            });
            ui.vertical(|ui| -> Result<(), ExpressionError> {
                "units:".cstr_c(ui.visuals().weak_text_color()).label(ui);
                for unit in &units {
                    context
                        .with_owner(unit.entity(), |context| {
                            (*unit).clone().see(context).tag_card(ui)
                        })
                        .ui(ui);
                }
                let statuses = context.collect_children_components::<NStatusAbility>(self.id)?;
                if !statuses.is_empty() {
                    "statuses:".cstr_c(ui.visuals().weak_text_color()).label(ui);
                    for status in statuses {
                        context
                            .with_owner_ref(status.entity(), |context| {
                                let charges = context.get_i32(VarName::charges).unwrap_or_default();
                                let color =
                                    context.get_color(VarName::color).unwrap_or(MISSING_COLOR);
                                if charges > 0 {
                                    TagWidget::new_name_value(
                                        &status.status_name,
                                        color,
                                        charges.cstr_s(CstrStyle::Bold),
                                    )
                                    .ui(ui);
                                }
                                Ok(())
                            })
                            .ui(ui);
                    }
                }
                "behavior:".cstr_c(ui.visuals().weak_text_color()).label(ui);
                let trigger = NFusion::get_trigger(context, &self.trigger)?;
                let vctx = ViewContext::new(ui).non_interactible(true);
                ui.horizontal(|ui| {
                    Icon::Lightning.show(ui);
                    trigger.view_title(vctx, context, ui);
                });

                let units = self.units(context)?;
                let unit_ids: Vec<u64> = units.iter().map(|u| u.id).collect();
                for (unit_index, ar) in self.behavior.iter().enumerate() {
                    if let Some(unit_id) = unit_ids.get(unit_index) {
                        for i in 0..ar.length as usize {
                            let action = NFusion::get_action(context, *unit_id, ar, i)?.clone();
                            context
                                .with_owner(context.entity(*unit_id)?, |context| {
                                    action.view_title(vctx, context, ui);
                                    Ok(())
                                })
                                .ui(ui);
                        }
                    }
                }
                Ok(())
            })
            .inner
        })
    }
}
