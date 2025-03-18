mod colorix;
mod cstr;
mod descriptions;
mod enum_colors;
mod ui;
mod utils;
mod widgets;

use bevy_egui::egui::menu::BarState;
pub use colorix::*;
pub use cstr::*;
pub use descriptions::*;
pub use enum_colors::*;
pub use ui::*;
pub use utils::*;
pub use widgets::*;

use ::utils::*;
use bevy::utils::hashbrown::HashMap;
use bevy::{
    app::{App, Plugin},
    prelude::{KeyCode, Mut, Resource, World},
};
use bevy_egui::egui;
use bevy_egui::egui::UiBuilder;
use colored::CustomColor;
use egui::{
    emath::Numeric, include_image, pos2, style::HandleShape, Align, Align2, Area, CollapsingHeader,
    Color32, ComboBox, CornerRadius, FontId, Frame, Id, Image, Key, Layout, Margin, NumExt, Order,
    Rect, Response, Sense, Shadow, Stroke, TextEdit, TextStyle, Ui, Widget, WidgetText,
};
use indexmap::IndexMap;
use itertools::Itertools;
use lerp::Lerp;
use schema::*;
use std::{cmp::Ordering, collections::VecDeque, mem, ops::RangeInclusive, sync::Mutex};
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
        bar_state.bar_menu(self, add_contents);
        bar_state.store(&self.ctx, bar_id);
    }
}
