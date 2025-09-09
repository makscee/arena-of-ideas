pub mod colorix;
pub mod descriptions;
pub mod enum_colors;
pub mod ui;
pub mod utils;

pub use colorix::*;
pub use descriptions::*;
pub use enum_colors::*;
pub use ui::*;
pub use utils::*;

use bevy_egui::egui;
use bevy_egui::egui::menu::BarState;
use colored::CustomColor;
use egui::{
    Align, Align2, Area, CollapsingHeader, Color32, ComboBox, CornerRadius, FontId, Frame, Id,
    Image, Key, Layout, Margin, NumExt, Order, Rect, Response, Sense, Shadow, Stroke, TextEdit,
    TextStyle, Ui, Widget, WidgetText, emath::Numeric, include_image, pos2, style::HandleShape,
};
use itertools::Itertools;
use parking_lot::Mutex;
use schema::*;
use std::cell::LazyCell;
use std::{cmp::Ordering, mem, ops::RangeInclusive};
use strum::IntoEnumIterator;
use utils_client::*;

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
        let mut bar_state = BarState::load(&self.ctx, bar_id);
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
