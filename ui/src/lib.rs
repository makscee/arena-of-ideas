mod cstr;
mod rarity;
mod show;
mod table_ext;
mod ui;
mod widgets;

use bevy::prelude::KeyCode;
use bevy::prelude::World;
use bevy::{
    app::{App, Plugin},
    prelude::{Mut, Resource},
};
use bevy_egui::egui;
use colored::CustomColor;
pub use cstr::*;
use egui::NumExt;
use egui::{include_image, pos2, Image, Ui};
use egui::{Align, Align2, Area, Id, Layout, Order};
use egui::{Color32, FontId, TextStyle};
use egui::{Frame, Margin, Rect, Rounding, Stroke};
use egui::{Response, Sense, Widget, WidgetText};
use indexmap::IndexMap;
use itertools::Itertools;
use lerp::Lerp;
pub use rarity::*;
pub use show::*;
use std::mem;
use std::sync::Mutex;
use strum::IntoEnumIterator;
use strum_macros::{AsRefStr, Display, EnumIter, FromRepr};
pub use ui::*;
use utils::*;
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
