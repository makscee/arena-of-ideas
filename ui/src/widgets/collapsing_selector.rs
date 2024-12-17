use egui::{CollapsingHeader, Shadow};

use super::*;

pub struct CollapsingSelector {
    pub name: WidgetText,
}

impl CollapsingSelector {
    pub fn new(name: impl Into<WidgetText>) -> Self {
        Self { name: name.into() }
    }
    pub fn ui<T: ToCstr + IntoEnumIterator + Clone + PartialEq>(
        value: &mut T,
        prefix: Option<&str>,
        ui: &mut Ui,
        replacer: impl FnOnce(&mut T, &mut T),
        content: impl FnOnce(&mut T, &mut Ui) -> bool,
    ) -> bool {
        const FRAME: Frame = Frame {
            inner_margin: Margin::same(4.0),
            outer_margin: Margin::same(4.0),
            rounding: Rounding::same(13.0),
            shadow: Shadow::NONE,
            fill: TRANSPARENT,
            stroke: STROKE_DARK,
        };
        FRAME
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let mut new_value = value.clone();
                    let r = Selector::new(prefix.unwrap_or_default()).ui_enum(&mut new_value, ui);
                    if r {
                        replacer(value, &mut new_value);
                        *value = new_value;
                    }
                    r || CollapsingHeader::new("")
                        .id_source(ui.next_auto_id())
                        .default_open(true)
                        .show_unindented(ui, |ui| content(value, ui))
                        .body_returned
                        .unwrap_or_default()
                })
                .inner
            })
            .inner
    }
}
