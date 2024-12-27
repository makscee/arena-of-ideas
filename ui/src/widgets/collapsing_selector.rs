use super::*;

pub struct CollapsingSelector;

impl CollapsingSelector {
    pub fn ui<T: ToCstr + IntoEnumIterator + Clone + PartialEq + Inject + Default + AsRef<str>>(
        value: &mut T,
        prefix: Option<&str>,
        ui: &mut Ui,
        content: impl FnOnce(&mut T, &mut Ui) -> bool,
    ) -> Response {
        const FRAME: Frame = Frame {
            inner_margin: Margin::same(2.0),
            outer_margin: Margin::same(2.0),
            rounding: Rounding::same(13.0),
            shadow: Shadow::NONE,
            fill: TRANSPARENT,
            stroke: STROKE_DARK,
        };
        let mut r = FRAME
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let mut new_value = value.clone();
                    let mut r =
                        Selector::new(prefix.unwrap_or_default()).ui_enum(&mut new_value, ui);
                    if r.changed() {
                        new_value.move_inner(value);
                        *value = new_value;
                    }
                    let cr = CollapsingHeader::new("")
                        .id_source(ui.next_auto_id())
                        .default_open(true)
                        .show_unindented(ui, |ui| content(value, ui));
                    r.changed = cr.body_returned.unwrap_or_default();
                    r
                })
                .inner
            })
            .inner;
        let rect = r.rect;
        let rect = rect.with_max_x(rect.min.x + 6.0);
        let resp = ui.allocate_rect(rect, Sense::click());
        let color = if resp.hovered() { YELLOW } else { VISIBLE_DARK };
        ui.painter()
            .rect_filled(rect.shrink2(egui::vec2(2.0, 13.0)), Rounding::ZERO, color);
        if resp.clicked() {
            r.changed = true;
            value.wrap();
        }
        r
    }
}
