mod cstr;
mod rarity;
mod ui;
mod utils;
mod widgets;

pub use cstr::*;
pub use rarity::*;
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
use colored::CustomColor;
use egui::{
    emath::Numeric, include_image, pos2, style::HandleShape, Align, Align2, Area, CollapsingHeader,
    Color32, ComboBox, FontId, Frame, Id, Image, Key, Layout, Margin, NumExt, Order, Rect,
    Response, Rounding, Sense, Shadow, Stroke, TextEdit, TextStyle, Ui, Widget, WidgetText,
};
use indexmap::IndexMap;
use itertools::Itertools;
use lerp::Lerp;
use schema::*;
use std::{cmp::Ordering, collections::VecDeque, mem, ops::RangeInclusive, sync::Mutex};
use strum::IntoEnumIterator;
use strum_macros::{AsRefStr, Display, EnumIter, FromRepr};
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
