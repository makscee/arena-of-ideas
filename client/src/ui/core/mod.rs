pub mod colorix;
pub mod cstr;
pub mod descriptions;
pub mod enum_colors;
pub mod ui;
pub mod utils;

use super::*;

use bevy_egui::egui::{Popup, UiKind, UiStackInfo, containers::menu::MenuConfig};
pub use colorix::*;
pub use cstr::*;
pub use descriptions::*;
pub use enum_colors::*;
pub use ui::*;
pub use utils::*;

pub trait ToCustomColor {
    fn to_custom_color(&self) -> CustomColor;
}

impl ToCustomColor for Color32 {
    fn to_custom_color(&self) -> CustomColor {
        let a = self.to_array();
        CustomColor::new(a[0], a[1], a[2])
    }
}

pub trait ResponseExt {
    fn show_menu(&self, ui: &mut Ui, content: impl FnOnce(&mut Ui));
}

impl ResponseExt for Response {
    fn show_menu(&self, ui: &mut Ui, content: impl FnOnce(&mut Ui)) {
        let config = MenuConfig::find(ui);
        Popup::menu(self)
            .close_behavior(config.close_behavior)
            .style(config.style.clone())
            .info(
                UiStackInfo::new(UiKind::Menu).with_tag_value(MenuConfig::MENU_CONFIG_TAG, config),
            )
            .show(content);
    }
}

const BAR_OPEN_FLAG_KEY: LazyCell<Id> = LazyCell::new(|| Id::new("bar open"));

pub trait UiExt {
    fn any_bar_open(&mut self) -> bool;
    fn key_down(&self, key: Key) -> bool;
}

impl UiExt for Ui {
    fn any_bar_open(&mut self) -> bool {
        self.ctx().get_frame_flag(*BAR_OPEN_FLAG_KEY)
    }

    fn key_down(&self, key: Key) -> bool {
        !self.ctx().wants_keyboard_input() && self.ctx().input(|r| r.key_pressed(key))
    }
}
