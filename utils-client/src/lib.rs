mod error;
mod game_timer;
mod operations;

use arboard::Clipboard;
pub use error::*;
pub use game_timer::*;
pub use operations::*;

/// Macro that expands to a match statement where each NodeKind variant maps to its corresponding type.
///
/// # Usage
///
/// ```
/// use utils_client::node_kind_match;
///
/// let kind = NodeKind::NUnit;
/// node_kind_match!(kind, {
///     // NodeType is now available as the specific type for this variant
///     world.entity_mut(entity).remove::<NodeType>();
/// });
/// ```
///
/// The macro creates a `NodeType` type alias within each match arm that corresponds to the
/// specific node type (e.g., `NUnit`, `NHouse`, etc.) for that `NodeKind` variant.
#[macro_export]
macro_rules! node_kind_match {
    ($kind:expr, $code:expr) => {
        match $kind {
            NodeKind::None => {
                unreachable!()
            }
            NodeKind::NArena => {
                type NodeType = NArena;
                $code
            }
            NodeKind::NFloorPool => {
                type NodeType = NFloorPool;
                $code
            }
            NodeKind::NFloorBoss => {
                type NodeType = NFloorBoss;
                $code
            }
            NodeKind::NPlayer => {
                type NodeType = NPlayer;
                $code
            }
            NodeKind::NPlayerData => {
                type NodeType = NPlayerData;
                $code
            }
            NodeKind::NPlayerIdentity => {
                type NodeType = NPlayerIdentity;
                $code
            }
            NodeKind::NHouse => {
                type NodeType = NHouse;
                $code
            }
            NodeKind::NHouseColor => {
                type NodeType = NHouseColor;
                $code
            }
            NodeKind::NAbilityMagic => {
                type NodeType = NAbilityMagic;
                $code
            }
            NodeKind::NAbilityDescription => {
                type NodeType = NAbilityDescription;
                $code
            }
            NodeKind::NAbilityEffect => {
                type NodeType = NAbilityEffect;
                $code
            }
            NodeKind::NStatusMagic => {
                type NodeType = NStatusMagic;
                $code
            }
            NodeKind::NStatusDescription => {
                type NodeType = NStatusDescription;
                $code
            }
            NodeKind::NStatusBehavior => {
                type NodeType = NStatusBehavior;
                $code
            }
            NodeKind::NStatusRepresentation => {
                type NodeType = NStatusRepresentation;
                $code
            }
            NodeKind::NStatusState => {
                type NodeType = NStatusState;
                $code
            }
            NodeKind::NTeam => {
                type NodeType = NTeam;
                $code
            }
            NodeKind::NBattle => {
                type NodeType = NBattle;
                $code
            }
            NodeKind::NMatch => {
                type NodeType = NMatch;
                $code
            }
            NodeKind::NFusion => {
                type NodeType = NFusion;
                $code
            }
            NodeKind::NFusionSlot => {
                type NodeType = NFusionSlot;
                $code
            }
            NodeKind::NUnit => {
                type NodeType = NUnit;
                $code
            }
            NodeKind::NUnitDescription => {
                type NodeType = NUnitDescription;
                $code
            }
            NodeKind::NUnitStats => {
                type NodeType = NUnitStats;
                $code
            }
            NodeKind::NUnitState => {
                type NodeType = NUnitState;
                $code
            }
            NodeKind::NUnitBehavior => {
                type NodeType = NUnitBehavior;
                $code
            }
            NodeKind::NUnitRepresentation => {
                type NodeType = NUnitRepresentation;
                $code
            }
        }
    };
}

use bevy::{math::vec2, prelude::*};
use bevy_egui::egui::{
    self, Color32, Id, Order, Pos2, Response, Stroke, TextureId, Ui, epaint::PathShape, pos2,
};
use parking_lot::{Mutex, MutexGuard};
use ron::ser::{PrettyConfig, to_string_pretty};
use schema::{VarName, VarValue};
use serde::Serialize;

static UNIT_PIXELS: Mutex<f32> = Mutex::new(10.0);
pub fn unit_pixels() -> f32 {
    *UNIT_PIXELS.lock()
}
pub fn set_unit_pixels(value: f32) {
    *UNIT_PIXELS.lock() = value;
}

pub const BEVY_MISSING_COLOR: LinearRgba = LinearRgba::new(1.0, 0.0, 1.0, 1.0);

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
pub fn draw_curve(
    p1: Pos2,
    p2: Pos2,
    p3: Pos2,
    p4: Pos2,
    width: f32,
    color: Color32,
    arrow: bool,
    ui: &mut Ui,
) {
    let points = [p1, p2, p3, p4];
    let stroke = Stroke { width, color };
    let curve = egui::Shape::CubicBezier(egui::epaint::CubicBezierShape::from_points_stroke(
        points,
        false,
        Color32::TRANSPARENT,
        stroke,
    ));
    ui.painter().add(curve);
    if !arrow {
        return;
    }
    let t = p4.to_vec2();
    let t1 = (p3.to_vec2() - t).normalized() * 15.0;
    let p1 = (t + t1 + t1.rot90()).to_pos2();
    let p2 = (t + t1 - t1.rot90()).to_pos2();
    let points = [p1, p4, p2];
    let arrow = egui::Shape::Path(PathShape::line(points.into(), stroke));
    ui.painter().add(arrow);
}
pub fn show_texture(size: f32, texture: TextureId, ui: &mut Ui) -> Response {
    ui.image(egui::load::SizedTexture::new(
        texture,
        egui::vec2(size, size),
    ))
}
pub fn world_to_screen_cam(pos: Vec3, cam: &Camera, cam_transform: &GlobalTransform) -> Vec2 {
    cam.world_to_viewport(cam_transform, pos)
        .unwrap_or_default()
}
pub fn screen_to_world_cam(pos: Vec2, cam: &Camera, cam_transform: &GlobalTransform) -> Vec2 {
    cam.viewport_to_world_2d(cam_transform, pos)
        .unwrap_or_default()
}
pub fn get_ctx_bool_id(ctx: &egui::Context, id: Id) -> Option<bool> {
    ctx.data(|r| r.get_temp::<bool>(id))
}
pub fn get_ctx_bool_id_default(ctx: &egui::Context, id: Id, d: bool) -> bool {
    ctx.data(|r| r.get_temp::<bool>(id).unwrap_or(d))
}
pub fn set_ctx_bool_id(ctx: &egui::Context, id: Id, value: bool) {
    ctx.data_mut(|w| w.insert_temp(id, value))
}
pub fn clear_ctx_bool_id(ctx: &egui::Context, id: Id) {
    ctx.data_mut(|w| w.remove_temp::<bool>(id));
}
pub fn get_ctx_bool(ctx: &egui::Context, key: &str) -> Option<bool> {
    get_ctx_bool_id(ctx, Id::new(key))
}
pub fn set_ctx_bool(ctx: &egui::Context, key: &str, value: bool) {
    set_ctx_bool_id(ctx, Id::new(key), value)
}
pub fn get_context_string(ctx: &egui::Context, key: &str) -> String {
    let id = Id::new(key);
    ctx.data(|r| r.get_temp::<String>(id).unwrap_or_default())
}
pub fn set_context_string(ctx: &egui::Context, key: &str, value: String) {
    let id = Id::new(key);
    ctx.data_mut(|w| w.insert_temp(id, value))
}
pub fn check_context_id(ctx: &egui::Context, key: &str, value: Id) -> bool {
    let id = Id::new(key);
    ctx.data(|r| {
        r.get_temp::<Id>(id)
            .and_then(|v| Some(v.eq(&value)))
            .unwrap_or_default()
    })
}
pub fn set_context_id(ctx: &egui::Context, key: &str, value: Id) {
    let id = Id::new(key);
    ctx.data_mut(|w| w.insert_temp(id, value))
}
pub fn clear_context_id(ctx: &egui::Context, key: &str) {
    let id = Id::new(key);
    ctx.data_mut(|w| w.remove::<Id>(id));
}
pub fn cursor_pos(world: &mut World) -> Option<Vec2> {
    world
        .query::<&bevy::window::Window>()
        .single(world)
        .ok()?
        .cursor_position()
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

pub trait CtxExt {
    fn bg_clicked(&self) -> Option<Pos2>;
    fn set_frame_flag(&self, id: impl Into<Id>);
    fn get_frame_flag(&self, id: impl Into<Id>) -> bool;
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
    fn set_frame_flag(&self, id: impl Into<Id>) {
        let frame = self.cumulative_pass_nr();
        self.data_mut(|w| w.insert_temp(id.into(), frame));
    }
    fn get_frame_flag(&self, id: impl Into<Id>) -> bool {
        let frame = self.cumulative_pass_nr();
        self.data(|r| r.get_temp::<u64>(id.into()).unwrap_or_default() + 1 >= frame)
    }
}

pub fn clipboard_get() -> Option<String> {
    Clipboard::new().and_then(|mut c| c.get_text()).ok()
}
pub fn clipboard_set(text: String) {
    log::info!("Clipboard set:\n{text}");
    Clipboard::new().unwrap().set_text(text).unwrap()
}
pub fn to_ron_string<T: Serialize>(value: &T) -> String {
    to_string_pretty(value, PrettyConfig::new().depth_limit(1)).unwrap()
}

pub trait F32toV2 {
    fn v2(self) -> egui::Vec2;
}

impl F32toV2 for f32 {
    fn v2(self) -> egui::Vec2 {
        egui::Vec2::splat(self)
    }
}
