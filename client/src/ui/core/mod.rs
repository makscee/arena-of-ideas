pub mod colorix;
pub mod cstr;
pub mod descriptions;
pub mod enum_colors;
pub mod ui;
pub mod utils;

use super::*;

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
    fn bar_menu(&self, add_contents: impl FnOnce(&mut Ui));
}

impl ResponseExt for Response {
    fn bar_menu(&self, add_contents: impl FnOnce(&mut Ui)) {
        let bar_id = self.id;
        let mut bar_state = egui::menu::BarState::load(&self.ctx, bar_id);
        bar_state.bar_menu(self, |ui| {
            ui.ctx().set_frame_flag(*BAR_OPEN_FLAG_KEY);
            add_contents(ui);
        });
        bar_state.store(&self.ctx, bar_id);
    }
}

const BAR_OPEN_FLAG_KEY: LazyCell<Id> = LazyCell::new(|| Id::new("bar open"));

pub trait UiExt {
    fn any_bar_open(&mut self) -> bool;
}

impl UiExt for Ui {
    fn any_bar_open(&mut self) -> bool {
        self.ctx().get_frame_flag(*BAR_OPEN_FLAG_KEY)
    }
}
