use super::*;

pub trait ViewCard: ViewFns {
    fn view_card(
        &self,
        context: &Context,
        vctx: ViewContext,
        ui: &mut Ui,
        rect: Rect,
    ) -> Result<(), ExpressionError>;
}

fn section(ui: &mut Ui, f: impl FnOnce(&mut Ui)) {
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
        f(ui);
    });
}

impl ViewCard for NUnit {
    fn view_card(
        &self,
        context: &Context,
        vctx: ViewContext,
        ui: &mut Ui,
        rect: Rect,
    ) -> Result<(), ExpressionError> {
        let color = context.color(ui);
        section(ui, |ui| {
            ui.vertical_centered_justified(|ui| {
                self.unit_name.cstr_cs(color, CstrStyle::Heading).label(ui);
            });
        });
        let description = self.description_load(context)?;
        let behavior = description.behavior_load(context)?;
        let rep = description.representation_load(context)?;
        section(ui, |ui| {
            let mut rect = ui.available_rect_before_wrap();
            rect.set_height(100.0);
            ui.set_clip_rect(rect);
            ui.expand_to_include_rect(rect);
            rect.max.y += rect.height();
            rect = rect.shrink(3.0);
            rep.paint(rect, context, ui).ui(ui);
            unit_rep().paint(rect, context, ui).ui(ui);
        });
        section(ui, |ui| {
            ui.horizontal_wrapped(|ui| -> Result<(), ExpressionError> {
                TagWidget::new_var_value(VarName::pwr, context.get_var(VarName::pwr)?).ui(ui);
                TagWidget::new_var_value(VarName::hp, context.get_var(VarName::hp)?).ui(ui);
                TagWidget::new_var_value(VarName::tier, (behavior.tier() as i32).into()).ui(ui);
                Ok(())
            })
            .inner
            .ui(ui);
        });
        section(ui, |ui| {
            behavior.show(context, ui);
        });
        section(ui, |ui| {
            description
                .description
                .cstr_c(ui.visuals().weak_text_color())
                .label_w(ui);
        });
        Ok(())
    }
}

impl ViewCard for NHouse {
    fn view_card(
        &self,
        context: &Context,
        vctx: ViewContext,
        ui: &mut Ui,
        rect: Rect,
    ) -> Result<(), ExpressionError> {
        let color = context.color(ui);
        section(ui, |ui| {
            ui.vertical_centered_justified(|ui| {
                self.house_name.cstr_cs(color, CstrStyle::Heading).label(ui);
            });
        });
        section(ui, |ui| {
            ui.vertical_centered_justified(|ui| -> Result<(), ExpressionError> {
                let ability = self.ability_magic_load(context)?;
                ability
                    .ability_name
                    .cstr_cs(color, CstrStyle::Heading2)
                    .label(ui);
                Ok(())
            })
            .inner
            .ui(ui);
        });
        section(ui, |ui| {
            ui.vertical_centered_justified(|ui| -> Result<(), ExpressionError> {
                let status = self.status_magic_load(context)?;
                status
                    .status_name
                    .cstr_cs(color, CstrStyle::Heading2)
                    .label(ui);
                Ok(())
            })
            .inner
            .ui(ui);
        });
        Ok(())
    }
}
