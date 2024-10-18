use super::*;

pub struct EnumSwitcher;

impl EnumSwitcher {
    pub fn show<E: Into<String> + IntoEnumIterator + Clone + PartialEq>(
        value: &mut E,
        ui: &mut Ui,
    ) -> bool {
        let mut clicked = false;
        ui.horizontal(|ui| {
            for e in E::iter() {
                if Button::click(e.clone())
                    .active(e.eq(value))
                    .ui(ui)
                    .clicked()
                {
                    clicked = true;
                    *value = e;
                }
            }
        });
        clicked
    }
}
