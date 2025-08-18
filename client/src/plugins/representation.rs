use super::*;

pub struct RepresentationPlugin;

impl Plugin for RepresentationPlugin {
    fn build(&self, _app: &mut App) {}
}

impl RepresentationPlugin {
    pub fn paint_rect(
        rect: Rect,
        context: &Context,
        m: &Material,
        ui: &mut Ui,
    ) -> Result<(), ExpressionError> {
        let mut p = Painter::new(rect, ui.ctx());
        if let Ok(color) = context.get_color(VarName::color) {
            p.color = color;
        }
        for a in &m.0 {
            a.paint(context, &mut p, ui)?
        }
        PainterAction::paint.paint(context, &mut p, ui)?;
        Ok(())
    }
}

pub trait MaterialPaint {
    fn paint(&self, rect: Rect, context: &Context, ui: &mut Ui) -> Result<(), ExpressionError>;
    fn pain_or_show_err(&self, rect: Rect, context: &Context, ui: &mut Ui);
}

impl MaterialPaint for Material {
    fn paint(&self, rect: Rect, context: &Context, ui: &mut Ui) -> Result<(), ExpressionError> {
        RepresentationPlugin::paint_rect(rect, context, self, ui)
    }
    fn pain_or_show_err(&self, rect: Rect, context: &Context, ui: &mut Ui) {
        match self.paint(rect, context, ui) {
            Ok(_) => {}
            Err(e) => {
                e.cstr().label(ui);
            }
        }
    }
}
