use super::*;

pub struct EnumSwitcher;

impl EnumSwitcher {
    pub fn show<E: ToCstr + IntoEnumIterator + Clone + PartialEq>(
        value: &mut E,
        ui: &mut Ui,
    ) -> bool {
        Self::show_iter(value, E::iter(), ui)
    }
    pub fn show_iter<E: ToCstr + Clone + PartialEq>(
        value: &mut E,
        iter: impl IntoIterator<Item = E>,
        ui: &mut Ui,
    ) -> bool {
        let mut clicked = false;
        ui.horizontal(|ui| {
            for e in iter {
                let c = e.cstr();
                if Button::click(c.get_text())
                    .cstr(c)
                    .active(e.eq(value))
                    .ui(ui)
                    .clicked()
                    && !e.eq(value)
                {
                    clicked = true;
                    *value = e;
                }
            }
        });
        clicked
    }
}
