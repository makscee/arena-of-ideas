mod cstr;
mod rarity;
mod table_ext;
mod ui;
mod utils;
mod widgets;

use ::utils::*;
use bevy::{
    app::{App, Plugin},
    prelude::{KeyCode, Mut, Resource, World},
};
use bevy_egui::egui;
use colored::CustomColor;
pub use cstr::*;
use egui::{
    include_image, pos2, Align, Align2, Area, Color32, FontId, Frame, Id, Image, Layout, Margin,
    NumExt, Order, Rect, Response, Rounding, Sense, Stroke, TextStyle, Ui, Widget, WidgetText,
};
use indexmap::IndexMap;
use itertools::Itertools;
use lerp::Lerp;
pub use rarity::*;
use schema::*;
use std::mem;
use std::sync::Mutex;
use strum::IntoEnumIterator;
use strum_macros::{AsRefStr, Display, EnumIter, FromRepr};
pub use ui::*;
pub use utils::*;
use utils_client::*;
pub use widgets::*;

pub trait ToCustomColor {
    fn to_custom_color(&self) -> CustomColor;
}

impl ToCustomColor for Color32 {
    fn to_custom_color(&self) -> CustomColor {
        let a = self.to_array();
        CustomColor::new(a[0], a[1], a[2])
    }
}
