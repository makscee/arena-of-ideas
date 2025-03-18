use super::*;

pub struct CollapsingSelector;

impl CollapsingSelector {
    pub fn ui<T: ToCstr + IntoEnumIterator + Clone + PartialEq + Inject + Default + AsRef<str>>(
        value: &mut T,
        prefix: Option<&str>,
        ui: &mut Ui,
        content: impl FnOnce(&mut T, &mut Ui) -> bool,
    ) -> bool {
        const FRAME: Frame = Frame {
            inner_margin: Margin::same(2),
            outer_margin: Margin::same(2),
            corner_radius: CornerRadius::same(13),
            shadow: Shadow::NONE,
            fill: TRANSPARENT,
            stroke: Stroke::NONE,
        };
        FRAME
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let mut new_value = value.clone();
                    let mut r =
                        Selector::new(prefix.unwrap_or_default()).ui_enum(&mut new_value, ui);
                    if r {
                        new_value.move_inner(value);
                        *value = new_value;
                    }
                    let cr = CollapsingHeader::new("")
                        .id_salt(ui.next_auto_id())
                        .default_open(true)
                        .show_unindented(ui, |ui| content(value, ui));
                    r |= cr.body_returned.unwrap_or_default();
                    r
                })
                .inner
            })
            .inner
    }
}
