use super::*;

pub struct EnumSwitcher {
    style: Option<CstrStyle>,
    columns: bool,
    prefix: Option<Cstr>,
}

impl EnumSwitcher {
    pub fn new() -> Self {
        Self {
            style: None,
            prefix: None,
            columns: false,
        }
    }
    pub fn style(mut self, style: CstrStyle) -> Self {
        self.style = Some(style);
        self
    }
    pub fn prefix(mut self, text: Cstr) -> Self {
        self.prefix = Some(text);
        self
    }
    pub fn columns(mut self) -> Self {
        self.columns = true;
        self
    }
    pub fn show<E: ToCstr + IntoEnumIterator + Clone + PartialEq>(
        self,
        value: &mut E,
        ui: &mut Ui,
    ) -> bool {
        self.show_iter(value, E::iter(), ui)
    }
    pub fn show_iter<E: ToCstr + Clone + PartialEq>(
        self,
        value: &mut E,
        iter: impl IntoIterator<Item = E>,
        ui: &mut Ui,
    ) -> bool {
        let mut clicked = false;
        fn modify_c(es: &EnumSwitcher, c: &mut Cstr) {
            if let Some(style) = es.style {
                *c = c.cstr_s(style);
            }
            if let Some(prefix) = es.prefix.clone() {
                *c = prefix + c;
            }
        }
        if self.columns {
            let iter = iter.into_iter();
            let len = iter.try_len().unwrap();
            if ui.available_width() < 30.0 {
                return false;
            }
            ui.columns(len, |ui| {
                for (i, e) in iter.enumerate() {
                    let mut c = e.cstr();
                    modify_c(&self, &mut c);
                    let active = e.eq(value);
                    ui[i].vertical_centered_justified(|ui| {
                        if Button::new(c).active(active).ui(ui).clicked() && !active {
                            clicked = true;
                            *value = e;
                        }
                    });
                }
            })
        } else {
            ui.horizontal(|ui| {
                for e in iter {
                    let mut c = e.cstr();
                    modify_c(&self, &mut c);
                    let active = e.eq(value);
                    if Button::new(c).active(active).ui(ui).clicked() && !active {
                        clicked = true;
                        *value = e;
                    }
                }
            });
        }
        clicked
    }
}
