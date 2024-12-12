mod error;
mod game_timer;
mod operations;
mod var_name;
mod var_value;

use bevy::prelude::*;
use bevy_egui::{
    egui::{self, pos2, Color32, Id, Order, Pos2, Response, TextureId, Ui},
    EguiContext,
};
use humanize_duration::prelude::DurationExt;
use std::{
    cmp::Ordering,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use bevy::math::vec2;
use chrono::Utc;
pub use error::*;
pub use game_timer::*;
pub use operations::*;
use serde::{Deserialize, Serialize};
use strum_macros::AsRefStr;
use strum_macros::{Display, EnumIter, EnumString};
pub use var_name::*;
pub use var_value::*;

pub const BEVY_MISSING_COLOR: LinearRgba = LinearRgba::new(1.0, 0.0, 1.0, 1.0);

#[inline]
pub fn default<T: Default>() -> T {
    Default::default()
}

pub fn name_color(_: &str) -> Color32 {
    Color32::PLACEHOLDER
}
pub fn just_pressed(key: KeyCode, world: &World) -> bool {
    world.resource::<ButtonInput<KeyCode>>().just_pressed(key)
}
pub fn just_pressed_key(world: &World) -> impl ExactSizeIterator<Item = &KeyCode> {
    world.resource::<ButtonInput<KeyCode>>().get_just_pressed()
}
pub fn left_mouse_pressed(world: &World) -> bool {
    world
        .resource::<ButtonInput<MouseButton>>()
        .pressed(MouseButton::Left)
}
pub fn left_mouse_just_pressed(world: &World) -> bool {
    world
        .resource::<ButtonInput<MouseButton>>()
        .just_pressed(MouseButton::Left)
}
pub fn left_mouse_just_released(world: &World) -> bool {
    world
        .resource::<ButtonInput<MouseButton>>()
        .just_released(MouseButton::Left)
}
pub fn right_mouse_just_pressed(world: &World) -> bool {
    world
        .resource::<ButtonInput<MouseButton>>()
        .just_pressed(MouseButton::Right)
}
pub fn right_mouse_just_released(world: &World) -> bool {
    world
        .resource::<ButtonInput<MouseButton>>()
        .just_released(MouseButton::Right)
}
pub fn egui_context(world: &mut World) -> Option<egui::Context> {
    world
        .query::<&mut EguiContext>()
        .get_single_mut(world)
        .map(|c| c.into_inner().get_mut().clone())
        .ok()
}
pub fn delta_time(world: &World) -> f32 {
    world.resource::<Time>().delta_seconds()
}
pub fn elapsed_seconds(world: &World) -> f32 {
    world.resource::<Time>().elapsed_seconds()
}
pub fn now_micros() -> i64 {
    Utc::now().timestamp_micros()
}
pub fn now_seconds() -> f64 {
    Utc::now().timestamp_millis() as f64 * 1000.0
}
pub fn get_ctx_bool_id(ctx: &egui::Context, id: Id) -> bool {
    ctx.data(|r| r.get_temp::<bool>(id).unwrap_or_default())
}
pub fn set_ctx_bool_id(ctx: &egui::Context, id: Id, value: bool) {
    ctx.data_mut(|w| w.insert_temp(id, value))
}
pub fn get_ctx_bool(ctx: &egui::Context, key: &str) -> bool {
    get_ctx_bool_id(ctx, Id::new(key))
}
pub fn set_ctx_bool(ctx: &egui::Context, key: &str, value: bool) {
    set_ctx_bool_id(ctx, Id::new(key), value)
}
pub fn get_ctx_bool_world(world: &mut World, key: &str) -> bool {
    let id = Id::new(key);
    get_ctx_bool_id_world(world, id)
}
pub fn set_ctx_bool_world(world: &mut World, key: &str, value: bool) {
    let id = Id::new(key);
    set_ctx_bool_id_world(world, id, value)
}
pub fn get_ctx_bool_id_world(world: &mut World, id: Id) -> bool {
    if let Some(ctx) = &egui_context(world) {
        get_ctx_bool_id(ctx, id)
    } else {
        default()
    }
}
pub fn set_ctx_bool_id_world(world: &mut World, id: Id, value: bool) {
    if let Some(ctx) = &egui_context(world) {
        set_ctx_bool_id(ctx, id, value);
    }
}
pub fn get_context_string(world: &mut World, key: &str) -> String {
    let id = Id::new(key);
    if let Some(context) = egui_context(world) {
        context.data(|r| r.get_temp::<String>(id).unwrap_or_default())
    } else {
        default()
    }
}
pub fn set_context_string(world: &mut World, key: &str, value: String) {
    let id = Id::new(key);
    if let Some(context) = egui_context(world) {
        context.data_mut(|w| w.insert_temp(id, value))
    }
}
pub fn check_context_id(world: &mut World, key: &str, value: Id) -> bool {
    let id = Id::new(key);
    if let Some(context) = egui_context(world) {
        context
            .data(|r| r.get_temp::<Id>(id).and_then(|v| Some(v.eq(&value))))
            .unwrap_or_default()
    } else {
        false
    }
}
pub fn set_context_id(world: &mut World, key: &str, value: Id) {
    let id = Id::new(key);
    if let Some(context) = egui_context(world) {
        context.data_mut(|w| w.insert_temp(id, value))
    }
}
pub fn clear_context_id(world: &mut World, key: &str) {
    let id = Id::new(key);
    if let Some(context) = egui_context(world) {
        context.data_mut(|w| w.remove::<Id>(id));
    }
}
pub fn cursor_pos(world: &mut World) -> Option<Vec2> {
    let window = world.query::<&bevy::window::Window>().single(world);
    window.cursor_position()
}
pub fn get_children(entity: Entity, world: &World) -> Vec<Entity> {
    world
        .get::<Children>(entity)
        .map(|c| c.to_vec())
        .unwrap_or_default()
}
pub fn get_children_recursive(entity: Entity, world: &World) -> Vec<Entity> {
    let mut children = get_children(entity, world);
    let mut i = 0;
    while i < children.len() {
        children.extend(get_children(children[i], world));
        i += 1;
    }
    children
}
pub fn get_parent(entity: Entity, world: &World) -> Option<Entity> {
    world.get::<Parent>(entity).map(|p| p.get())
}
// pub fn copy_to_clipboard(text: &str, world: &mut World) {
//     world
//         .resource_mut::<bevy_egui::EguiClipboard>()
//         .set_contents(text);
//     debug!("Saved to clipboard:\n{text}");
// }
// pub fn paste_from_clipboard(world: &mut World) -> Option<String> {
//     world
//         .resource_mut::<bevy_egui::EguiClipboard>()
//         .get_contents()
// }

pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let x = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}
pub fn format_timestamp(ts: u64) -> String {
    if ts == 0 {
        return "-".into();
    }
    let d = SystemTime::now().duration_since(UNIX_EPOCH).unwrap() - Duration::from_micros(ts);
    format!(
        "{} ago",
        d.human(humanize_duration::Truncate::Minute).to_string()
    )
}
pub fn format_duration(seconds: u64) -> String {
    Duration::from_secs(seconds)
        .human(humanize_duration::Truncate::Second)
        .to_string()
}
pub fn show_texture(size: f32, texture: TextureId, ui: &mut Ui) -> Response {
    ui.image(egui::load::SizedTexture::new(
        texture,
        egui::vec2(size, size),
    ))
}

pub trait ToBVec2 {
    fn to_bvec2(&self) -> Vec2;
}

impl ToBVec2 for Pos2 {
    fn to_bvec2(&self) -> Vec2 {
        vec2(self.x, self.y)
    }
}

pub trait ToPos2 {
    fn to_pos2(&self) -> Pos2;
    fn to_evec2(&self) -> egui::Vec2;
}

impl ToPos2 for Vec2 {
    fn to_pos2(&self) -> Pos2 {
        pos2(self.x, self.y)
    }
    fn to_evec2(&self) -> egui::Vec2 {
        egui::vec2(self.x, self.y)
    }
}

pub trait ToColor {
    fn to_color(&self) -> Color;
}

impl ToColor for Color32 {
    fn to_color(&self) -> Color {
        let a = self.to_array();
        Color::srgba_u8(a[0], a[1], a[2], a[3])
    }
}

pub trait ToC32 {
    fn c32(&self) -> Color32;
}

impl ToC32 for Color {
    fn c32(&self) -> Color32 {
        let c = self.to_srgba().to_u8_array();
        Color32::from_rgba_unmultiplied(c[0], c[1], c[2], c[3])
    }
}

pub trait CtxExt {
    fn bg_clicked(&self) -> Option<Pos2>;
}

impl CtxExt for egui::Context {
    fn bg_clicked(&self) -> Option<Pos2> {
        if !self.input(|r| r.pointer.primary_clicked()) {
            return None;
        }
        let Some(pos) = self.pointer_interact_pos() else {
            return None;
        };
        if self.available_rect().contains(pos)
            && self
                .layer_id_at(pos)
                .map(|l| l.order == Order::Background)
                .unwrap_or(true)
        {
            Some(pos)
        } else {
            None
        }
    }
}
