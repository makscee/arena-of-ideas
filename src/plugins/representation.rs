use super::*;

pub struct RepresentationPlugin;

impl Plugin for RepresentationPlugin {
    fn build(&self, _app: &mut App) {}
}

impl RepresentationPlugin {
    fn paint(
        m: &Material,
        context: &Context,
        ctx: &egui::Context,
        cam: (&Camera, &GlobalTransform),
    ) -> Result<(), ExpressionError> {
        let pos = context.get_var(VarName::position)?.get_vec2()?
            + context
                .get_var(VarName::offset)
                .and_then(|v| v.get_vec2())
                .unwrap_or_default();
        let pos = world_to_screen_cam(pos.extend(0.0), &cam.0, &cam.1).to_pos2();
        let size = unit_pixels() * 2.1;
        let size = egui::vec2(size, size);
        let rect = Rect::from_center_size(pos, size);
        if !ctx.screen_rect().intersects(rect) {
            return Ok(());
        }
        let owner = context.owner_entity()?;
        Area::new(Id::new(owner))
            .constrain(false)
            .fixed_pos(rect.center())
            .pivot(Align2::CENTER_CENTER)
            .order(Order::Background)
            .show(ctx, |ui| Self::paint_rect(rect, context, m, ui))
            .inner
    }
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

impl NRepresentation {
    pub fn paint(&self, rect: Rect, context: &Context, ui: &mut Ui) -> Result<(), ExpressionError> {
        RepresentationPlugin::paint_rect(rect, context, &self.material, ui)
    }
    pub fn pain_or_show_err(&self, rect: Rect, context: &Context, ui: &mut Ui) {
        match self.paint(rect, context, ui) {
            Ok(_) => {}
            Err(e) => {
                e.cstr().label(ui);
            }
        }
    }
}
