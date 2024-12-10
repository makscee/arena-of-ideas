use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, AsRefStr, EnumIter, PartialEq, Eq)]
pub enum Trigger {
    #[default]
    BattleStart,
    TurnEnd,
}

impl ToCstr for Trigger {
    fn cstr(&self) -> Cstr {
        self.as_ref().to_owned()
    }
}

impl Show for Trigger {
    fn show(&self, prefix: Option<&str>, ui: &mut bevy_egui::egui::Ui) {
        if let Some(prefix) = prefix {
            prefix.cstr_c(VISIBLE_DARK).label(ui);
        }
        self.cstr_cs(CYAN, CstrStyle::Bold).label(ui);
    }
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut bevy_egui::egui::Ui) -> bool {
        Selector::new(prefix.unwrap_or_default()).ui_enum(self, ui)
    }
}
