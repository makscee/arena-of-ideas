use super::*;

#[derive(Clone, Copy, Default)]
pub struct TagCardContext {
    expanded: bool,
}

impl TagCardContext {
    pub fn new() -> Self {
        default()
    }
    pub fn expanded(mut self, value: bool) -> Self {
        self.expanded = value;
        self
    }
    fn merge_state(mut self, node: &impl Node, ui: &mut Ui) -> Self {
        if let Some(other) = ui
            .ctx()
            .data(|r| r.get_temp::<TagCardContext>(node.egui_id().with(ui.id())))
        {
            self.expanded = other.expanded;
        }
        self
    }
    fn save(self, node: &impl Node, ui: &mut Ui) {
        ui.ctx()
            .data_mut(|w| w.insert_temp(node.egui_id().with(ui.id()), self));
    }
}

fn show_frame(
    node: &impl ViewFns,
    name: &str,
    color: Color32,
    context: &Context,
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
                                node.view(ViewContext::new(ui), context, ui);
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

pub trait TagCard: Node {
    fn tag_card(
        &self,
        tctx: TagCardContext,
        context: &Context,
        ui: &mut Ui,
    ) -> Result<(), ExpressionError> {
        let tctx = tctx.merge_state(self, ui);
        context.with_layer_ref_r(ContextLayer::Owner(self.entity()), |context| {
            let response = if tctx.expanded {
                self.show_card(context, ui)?
            } else {
                self.show_tag(context, ui)?
            };
            if response.clicked() {
                tctx.expanded(!tctx.expanded).save(self, ui);
            }
            Ok(())
        })
    }
    fn show_tag(&self, context: &Context, ui: &mut Ui) -> Result<Response, ExpressionError>;
    fn show_card(&self, context: &Context, ui: &mut Ui) -> Result<Response, ExpressionError>;
}

impl TagCard for NUnit {
    fn show_tag(&self, context: &Context, ui: &mut Ui) -> Result<Response, ExpressionError> {
        let tier = if let Ok(behavior) = context.first_parent_recursive::<NUnitBehavior>(self.id) {
            behavior.reactions.tier()
        } else {
            0
        };
        let lvl = context.get_i32(VarName::lvl).unwrap_or_default();
        let xp = match context.get_i32(VarName::xp) {
            Ok(v) => format!(" [tw {v}]/[{} [b {lvl}]]", VarName::lvl.color().to_hex()),
            Err(_) => default(),
        };
        Ok(TagWidget::new_name_value(
            context.get_string(VarName::unit_name)?,
            context.get_color(VarName::color)?,
            format!(
                "[b {} {} [tw T]{}]{xp}",
                context.get_i32(VarName::pwr)?.cstr_c(VarName::pwr.color()),
                context.get_i32(VarName::hp)?.cstr_c(VarName::hp.color()),
                (tier as i32).cstr_c(VarName::tier.color())
            ),
        )
        .ui(ui))
    }
    fn show_card(&self, context: &Context, ui: &mut Ui) -> Result<Response, ExpressionError> {
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
impl TagCard for NHouse {
    fn show_tag(&self, context: &Context, ui: &mut Ui) -> Result<Response, ExpressionError> {
        let color = context.color(ui);
        Ok(TagWidget::new_name(&self.house_name, color).ui(ui))
    }
    fn show_card(&self, context: &Context, ui: &mut Ui) -> Result<Response, ExpressionError> {
        let color = context.color(ui);
        Ok(show_frame(
            self,
            &self.house_name,
            color,
            context,
            ui,
            |ui| {
                if let Ok(action) = self.action_load(context) {
                    action.tag_card(default(), context, ui).ui(ui);
                }
                if let Ok(status) = self.status_load(context) {
                    status.tag_card(default(), context, ui).ui(ui);
                }
                for unit in self.units_load(context) {
                    context
                        .with_owner_ref(unit.entity(), |context| {
                            unit.tag_card(default(), context, ui)
                        })
                        .ui(ui);
                }
            },
        ))
    }
}
impl TagCard for NActionAbility {
    fn show_tag(&self, context: &Context, ui: &mut Ui) -> Result<Response, ExpressionError> {
        let color = context.color(ui);
        Ok(TagWidget::new_name(&self.ability_name, color).ui(ui))
    }
    fn show_card(&self, context: &Context, ui: &mut Ui) -> Result<Response, ExpressionError> {
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
impl TagCard for NStatusAbility {
    fn show_tag(&self, context: &Context, ui: &mut Ui) -> Result<Response, ExpressionError> {
        let color = context.color(ui);
        Ok(TagWidget::new_name(&self.status_name, color).ui(ui))
    }
    fn show_card(&self, context: &Context, ui: &mut Ui) -> Result<Response, ExpressionError> {
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
                            unit.tag_card(default(), context, ui)
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
                for (tr, actions) in &self.behavior {
                    if actions.is_empty() {
                        continue;
                    }
                    let trigger = NFusion::get_trigger(context, tr)?;
                    let vctx = ViewContext::new(ui).non_interactible(true);
                    ui.horizontal(|ui| {
                        Icon::Lightning.show(ui);
                        trigger.view_title(vctx, context, ui);
                    });
                    for ar in actions {
                        let action = NFusion::get_action(context, ar)?.clone();
                        context
                            .with_owner(context.entity(ar.unit)?, |context| {
                                action.view_title(vctx, context, ui);
                                Ok(())
                            })
                            .ui(ui);
                    }
                }
                Ok(())
            })
            .inner
        })
    }
}
