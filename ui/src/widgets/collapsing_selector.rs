use egui::{CollapsingHeader, Shadow};

use super::*;

pub struct CollapsingSelector {
    pub name: WidgetText,
}

impl CollapsingSelector {
    pub fn new(name: impl Into<WidgetText>) -> Self {
        Self { name: name.into() }
    }
    pub fn ui<T: ToCstr + IntoEnumIterator + Clone + PartialEq + Inject + Default>(
        value: &mut T,
        prefix: Option<&str>,
        ui: &mut Ui,
        content: impl FnOnce(&mut T, &mut Ui) -> bool,
    ) -> bool {
        const FRAME: Frame = Frame {
            inner_margin: Margin::same(2.0),
            outer_margin: Margin::same(2.0),
            rounding: Rounding::same(13.0),
            shadow: Shadow::NONE,
            fill: TRANSPARENT,
            stroke: STROKE_DARK,
        };
        let resp = FRAME.show(ui, |ui| {
            ui.horizontal(|ui| {
                let mut new_value = value.clone();
                let r = Selector::new(prefix.unwrap_or_default()).ui_enum(&mut new_value, ui);
                if r {
                    new_value.move_inner(value);
                    *value = new_value;
                }
                let cr = CollapsingHeader::new("")
                    .id_source(ui.next_auto_id())
                    .default_open(true)
                    .show_unindented(ui, |ui| content(value, ui));

                cr.body_returned.unwrap_or_default() || r
            })
            .inner
        });
        let mut r = resp.inner;
        let rect = resp.response.rect;
        let rect = rect.with_max_x(rect.min.x + 6.0);
        let resp = ui.allocate_rect(rect, Sense::click());
        let color = if resp.hovered() { YELLOW } else { VISIBLE_DARK };
        ui.painter()
            .rect_filled(rect.shrink2(egui::vec2(2.0, 13.0)), Rounding::ZERO, color);
        if resp.clicked() {
            r = true;
            value.wrap();
        }
        r
    }
}
