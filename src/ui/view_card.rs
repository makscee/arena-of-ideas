use super::*;

pub trait ViewCard: ViewFns {
    fn view_card(&self, context: &Context, ui: &mut Ui) -> Result<Response, ExpressionError> {
        let mut rect = ui.available_rect_before_wrap();
        let resp = ui
            .allocate_rect(rect, Sense::click_and_drag())
            .on_hover_ui_at_pointer(|ui| {
                self.show_card_on_hover(context, ui).ui(ui);
            });
        let t = ui
            .ctx()
            .animate_bool(resp.id, resp.hovered() && !resp.is_pointer_button_down_on());
        rect = rect.expand(t * 4.0);
        let mut translation = egui::Vec2::ZERO;
        if resp.dragged() {
            if let Some(pos) = ui.ctx().input(|r| r.pointer.latest_pos()) {
                translation = pos - rect.center();
            }
        }
        ui.with_visual_transform(
            TSTransform::from_translation(translation),
            |ui| -> Result<(), ExpressionError> {
                let ui = &mut ui.new_child(UiBuilder::new().max_rect(rect));
                ui.set_clip_rect(rect);
                let margin: Margin = 2.into();
                Frame::new()
                    .stroke(GRAY.stroke())
                    .inner_margin(margin)
                    .corner_radius(6)
                    .show(ui, |ui| self.show_card_sections(context, ui))
                    .inner
            },
        )
        .inner?;
        Ok(resp)
    }
    fn show_card_sections(&self, context: &Context, ui: &mut Ui) -> Result<(), ExpressionError>;
    fn show_card_on_hover(&self, context: &Context, ui: &mut Ui) -> Result<(), ExpressionError>;
}

fn section(ui: &mut Ui, f: impl FnOnce(&mut Ui) -> Result<(), ExpressionError>) {
    Frame {
        inner_margin: Margin::same(4),
        outer_margin: Margin::same(1),
        fill: ui.visuals().widgets.inactive.bg_fill,
        stroke: Stroke::NONE,
        corner_radius: CornerRadius::same(6),
        shadow: Shadow::NONE,
    }
    .show(ui, |ui| {
        ui.expand_to_include_x(ui.available_rect_before_wrap().max.x);
        f(ui).ui(ui);
    });
}

impl ViewCard for NUnit {
    fn show_card_sections(&self, context: &Context, ui: &mut Ui) -> Result<(), ExpressionError> {
        let color = context.color(ui);
        ui.with_layout(
            Layout::bottom_up(Align::Center).with_main_wrap(true),
            |ui| -> Result<(), ExpressionError> {
                let description = self.description_load(context)?;
                let behavior = description.behavior_load(context)?;
                let rep = description.representation_load(context)?;
                section(ui, |ui| {
                    description
                        .description
                        .cstr_cs(ui.visuals().weak_text_color(), CstrStyle::Small)
                        .label_w(ui);
                    Ok(())
                });
                const NAME_HEIGHT: f32 = 35.0;
                let mut rect = ui.available_rect_before_wrap();
                {
                    ui.set_clip_rect(rect);
                    ui.expand_to_include_rect(rect);
                    rect.max.y += rect.height();
                    rect = rect.shrink(3.0);
                    rep.paint(rect, context, ui).ui(ui);
                    unit_rep().paint(rect, context, ui).ui(ui);
                }
                {
                    rect.min.y += NAME_HEIGHT;
                    let ui = &mut ui.new_child(UiBuilder::new().max_rect(rect));
                    ui.vertical(|ui| -> Result<(), ExpressionError> {
                        TagWidget::new_var_value(VarName::pwr, context.get_var(VarName::pwr)?)
                            .ui(ui);
                        TagWidget::new_var_value(VarName::hp, context.get_var(VarName::hp)?).ui(ui);
                        TagWidget::new_var_value(VarName::tier, (behavior.tier() as i32).into())
                            .ui(ui);
                        Ok(())
                    })
                    .inner
                    .ui(ui);
                }
                let mut rect = ui.available_rect_before_wrap();
                rect.set_height(NAME_HEIGHT);
                let ui = &mut ui.new_child(UiBuilder::new().max_rect(rect));
                ui.vertical_centered_justified(|ui| {
                    self.unit_name.cstr_cs(color, CstrStyle::Heading).label(ui);
                });
                Ok(())
            },
        )
        .inner
    }
    fn show_card_on_hover(&self, context: &Context, ui: &mut Ui) -> Result<(), ExpressionError> {
        let behavior = self.description_load(context)?.behavior_load(context)?;
        section(ui, |ui| {
            behavior.show(context, ui);
            Ok(())
        });
        Ok(())
    }
}

impl ViewCard for NHouse {
    fn show_card_sections(&self, context: &Context, ui: &mut Ui) -> Result<(), ExpressionError> {
        let color = context.color(ui);
        section(ui, |ui| {
            ui.vertical_centered_justified(|ui| {
                self.house_name.cstr_cs(color, CstrStyle::Heading).label(ui);
            });
            Ok(())
        });
        section(ui, |ui| {
            let ability = self.ability_magic_load(context)?;
            ability
                .ability_name
                .cstr_cs(color, CstrStyle::Heading2)
                .label(ui);
            ability
                .description_load(context)?
                .description
                .cstr_c(ui.visuals().weak_text_color())
                .label_w(ui);
            Ok(())
        });
        section(ui, |ui| -> Result<(), ExpressionError> {
            let status = self.status_magic_load(context)?;
            status
                .status_name
                .cstr_cs(color, CstrStyle::Heading2)
                .label(ui);
            status
                .description_load(context)?
                .description
                .cstr_c(ui.visuals().weak_text_color())
                .label_w(ui);
            Ok(())
        });
        Ok(())
    }
    fn show_card_on_hover(&self, context: &Context, ui: &mut Ui) -> Result<(), ExpressionError> {
        let color = context.color(ui);
        section(ui, |ui| {
            let ability = self.ability_magic_load(context)?;
            ability
                .ability_name
                .cstr_cs(color, CstrStyle::Heading2)
                .label(ui);
            ability
                .description_load(context)?
                .effect_load(context)?
                .show(context, ui);
            Ok(())
        });
        section(ui, |ui| -> Result<(), ExpressionError> {
            let status = self.status_magic_load(context)?;
            status
                .status_name
                .cstr_cs(color, CstrStyle::Heading2)
                .label(ui);
            status
                .description_load(context)?
                .behavior_load(context)?
                .show(context, ui);
            Ok(())
        });
        Ok(())
    }
}
