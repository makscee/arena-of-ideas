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
    node: &impl DataView,
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
        let context = &context.clone().set_owner(self.entity()).take();
        let response = if tctx.expanded {
            self.show_card(context, ui)?
        } else {
            self.show_tag(context, ui)?
        };
        if response.clicked() {
            tctx.expanded(!tctx.expanded).save(self, ui);
        }
        Ok(())
    }
    fn show_tag(&self, context: &Context, ui: &mut Ui) -> Result<Response, ExpressionError>;
    fn show_card(&self, context: &Context, ui: &mut Ui) -> Result<Response, ExpressionError>;
}

impl TagCard for NUnit {
    fn show_tag(&self, context: &Context, ui: &mut Ui) -> Result<Response, ExpressionError> {
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
    fn show_card(&self, context: &Context, ui: &mut Ui) -> Result<Response, ExpressionError> {
        let color = context.color(ui);
        let pwr = context.get_var(VarName::pwr)?;
        let hp = context.get_var(VarName::hp)?;
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
                });
                if let Some(description) = self.description_load(context) {
                    description.description.label_w(ui);
                    if let Some(behavior) = description.behavior_load(context) {
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
                if let Some(ability) = self.ability_magic_load(context) {
                    ability.tag_card(default(), context, ui).ui(ui);
                }
                if let Some(status) = self.status_magic_load(context) {
                    status.tag_card(default(), context, ui).ui(ui);
                }
                for unit in self.units_load(context) {
                    unit.tag_card(default(), context.clone().set_owner(unit.entity()), ui)
                        .ui(ui);
                }
            },
        ))
    }
}
impl TagCard for NAbilityMagic {
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
                if let Some(description) = self.description_load(context) {
                    description.description.label_w(ui);
                    if let Some(effect) = description.effect_load(context) {
                        effect.show(context, ui);
                    }
                }
            },
        ))
    }
}
impl TagCard for NStatusMagic {
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
                if let Some(description) = self.description_load(context) {
                    description.description.label_w(ui);
                    if let Some(behavior) = description.behavior_load(context) {
                        behavior.show(context, ui);
                    }
                }
            },
        ))
    }
}
impl NFusion {
    pub fn show_card(&self, context: &Context, ui: &mut Ui) -> Result<(), ExpressionError> {
        let units = self.units(context)?;
        let context = &context.clone().set_owner(self.entity()).take();
        let pwr = context.get_var(VarName::pwr)?;
        let hp = context.get_var(VarName::hp)?;
        let statuses = context.children_components::<NStatusMagic>(self.entity());

        ui.horizontal(|ui| {
            TagWidget::new_var_value(VarName::pwr, pwr).ui(ui);
            TagWidget::new_var_value(VarName::hp, hp).ui(ui);
        });
        ui.vertical(|ui| -> Result<(), ExpressionError> {
            "units:".cstr_c(ui.visuals().weak_text_color()).label(ui);
            for unit in &units {
                unit.tag_card(default(), context.clone().set_owner(unit.entity()), ui)
                    .ui(ui);
            }
            if !statuses.is_empty() {
                "statuses:".cstr_c(ui.visuals().weak_text_color()).label(ui);
                for status in statuses {
                    let context = context.clone().set_owner(status.entity()).take();
                    let charges = context.get_i32(VarName::charges).unwrap_or_default();
                    let color = context.get_color(VarName::color).unwrap_or(MISSING_COLOR);
                    if charges > 0 {
                        TagWidget::new_name_value(
                            &status.status_name,
                            color,
                            charges.cstr_s(CstrStyle::Bold),
                        )
                        .ui(ui);
                    }
                }
            }
            "behavior:".cstr_c(ui.visuals().weak_text_color()).label(ui);
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
        .inner
    }
}
