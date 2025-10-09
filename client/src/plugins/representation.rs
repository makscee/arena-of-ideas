use super::*;

pub struct RepresentationPlugin;

impl Plugin for RepresentationPlugin {
    fn build(&self, _app: &mut App) {}
}

impl RepresentationPlugin {
    pub fn paint_rect(
        rect: Rect,
        context: &ClientContext,
        m: &Material,
        ui: &mut Ui,
    ) -> NodeResult<()> {
        let mut p = Painter::new(rect, ui.ctx());
        if let Ok(color) = context.get_var(VarName::color).get_color() {
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
    fn paint(&self, rect: Rect, context: &ClientContext, ui: &mut Ui) -> NodeResult<()>;
    fn paint_viewer(&self, context: &ClientContext, ui: &mut Ui) -> Response;
}

impl MaterialPaint for Material {
    fn paint(&self, rect: Rect, context: &ClientContext, ui: &mut Ui) -> NodeResult<()> {
        RepresentationPlugin::paint_rect(rect, context, self, ui)
    }
    fn paint_viewer(&self, context: &ClientContext, ui: &mut Ui) -> Response {
        let size_id = ui.id().with("view size");
        let mut size = ui.ctx().data_mut(|w| *w.get_temp_mut_or(size_id, 100.0));
        if DragValue::new(&mut size).ui(ui).changed() {
            size = size.at_least(10.0);
            ui.ctx().data_mut(|w| w.insert_temp(size_id, size));
        }
        let (rect, response) = ui.allocate_exact_size(egui::vec2(size, size), Sense::hover());
        self.paint(rect, context, ui).ui(ui);
        ui.painter().rect_stroke(
            rect,
            0,
            Stroke::new(1.0, subtle_borders_and_separators()),
            egui::StrokeKind::Middle,
        );
        response
    }
}
