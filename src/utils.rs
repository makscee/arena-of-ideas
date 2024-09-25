use bevy::{color::ColorToPacked, input::mouse::MouseButton};
use chrono::Utc;
use spacetimedb_sdk::table::TableType;

use super::*;

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
pub fn get_context_bool(world: &mut World, key: &str) -> bool {
    let id = Id::new(key);
    get_context_bool_id(world, id)
}
pub fn set_context_bool(world: &mut World, key: &str, value: bool) {
    let id = Id::new(key);
    set_context_bool_id(world, id, value)
}
pub fn get_context_bool_id(world: &mut World, id: Id) -> bool {
    if let Some(context) = egui_context(world) {
        context.data(|r| r.get_temp::<bool>(id).unwrap_or_default())
    } else {
        default()
    }
}
pub fn set_context_bool_id(world: &mut World, id: Id, value: bool) {
    if let Some(context) = egui_context(world) {
        context.data_mut(|w| w.insert_temp(id, value))
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
pub fn world_to_screen(pos: Vec3, world: &World) -> Vec2 {
    let entity = world.entity(world.resource::<CameraData>().entity);
    let camera = entity.get::<Camera>().unwrap();
    let transform = entity.get::<GlobalTransform>().unwrap();
    camera.world_to_viewport(transform, pos).unwrap_or_default()
}
pub fn screen_to_world(pos: Vec2, world: &World) -> Vec2 {
    let entity = CameraPlugin::entity(world);
    let camera = world.get::<Camera>(entity).unwrap();
    let transform = world.get::<GlobalTransform>(entity).unwrap();
    screen_to_world_cam(pos, camera, transform)
}
pub fn screen_to_world_cam(pos: Vec2, camera: &Camera, transform: &GlobalTransform) -> Vec2 {
    camera.viewport_to_world_2d(transform, pos).unwrap()
}
pub fn cursor_pos(world: &mut World) -> Option<Vec2> {
    let window = world.query::<&bevy::window::Window>().single(world);
    window.cursor_position()
}
pub fn cursor_world_pos(world: &mut World) -> Option<Vec2> {
    cursor_pos(world).map(|p| screen_to_world(p, world))
}
pub fn get_children(entity: Entity, world: &World) -> Vec<Entity> {
    world
        .get::<Children>(entity)
        .map(|c| c.to_vec())
        .unwrap_or_default()
}
pub fn save_to_clipboard(text: &str, world: &mut World) {
    world
        .resource_mut::<bevy_egui::EguiClipboard>()
        .set_contents(text);
    debug!("Saved to clipboard:\n{text}");
}
pub fn get_from_clipboard(world: &mut World) -> Option<String> {
    world
        .resource_mut::<bevy_egui::EguiClipboard>()
        .get_contents()
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
pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let x = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}
pub fn format_timestamp(ts: u64) -> String {
    DateTime::<chrono::Local>::from(UNIX_EPOCH + Duration::from_micros(ts))
        .format("%d/%m/%Y %H:%M")
        .to_string()
}
pub fn global_settings() -> GlobalSettings {
    GlobalSettings::find_by_always_zero(0).unwrap()
}
pub fn app_exit(world: &mut World) {
    world
        .get_resource_mut::<bevy::prelude::Events<bevy::app::AppExit>>()
        .unwrap()
        .send(bevy::app::AppExit::Success);
}
pub fn app_exit_op() {
    OperationsPlugin::add(app_exit)
}
pub fn cur_state(world: &World) -> GameState {
    *world.resource::<State<GameState>>().get()
}
pub fn debug_rect(rect: Rect, ctx: &egui::Context) {
    ctx.debug_painter().rect(
        rect,
        Rounding::ZERO,
        YELLOW_DARK,
        Stroke {
            width: 1.0,
            color: YELLOW,
        },
    );
}
pub fn can_afford(cost: i64) -> bool {
    TWallet::current().amount >= cost
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
}

impl ToPos2 for Vec2 {
    fn to_pos2(&self) -> Pos2 {
        pos2(self.x, self.y)
    }
}

pub trait EntityExt {
    fn get_parent(self, world: &World) -> Option<Entity>;
    fn get_parent_query(self, query: &Query<&Parent>) -> Option<Entity>;
    fn faction(self, world: &World) -> Faction;
}

impl EntityExt for Entity {
    fn get_parent(self, world: &World) -> Option<Entity> {
        world.get::<Parent>(self).map(|p| p.get())
    }
    fn get_parent_query(self, query: &Query<&Parent>) -> Option<Entity> {
        query.get(self).ok().map(|p| p.get())
    }
    fn faction(self, world: &World) -> Faction {
        Context::new(self)
            .get_value(VarName::Faction, world)
            .unwrap()
            .get_faction()
            .unwrap()
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

pub trait ToCustomColor {
    fn to_custom_color(&self) -> CustomColor;
}

impl ToCustomColor for Color32 {
    fn to_custom_color(&self) -> CustomColor {
        let a = self.to_array();
        CustomColor::new(a[0], a[1], a[2])
    }
}

pub trait WorldExt {
    fn game_clear(&mut self);
}

impl WorldExt for World {
    fn game_clear(&mut self) {
        Representation::despawn_all(self);
        clear_entity_names();
    }
}

pub trait TableSingletonExt {
    fn current() -> Self;
    fn get_current() -> Option<Box<Self>>;
}

impl TableSingletonExt for GlobalData {
    fn current() -> Self {
        *Self::get_current().unwrap()
    }
    fn get_current() -> Option<Box<Self>> {
        Self::iter().exactly_one().ok().map(|d| Box::new(d))
    }
}
impl TableSingletonExt for GlobalSettings {
    fn current() -> Self {
        *Self::get_current().unwrap()
    }
    fn get_current() -> Option<Box<Self>> {
        Self::iter().exactly_one().ok().map(|d| Box::new(d))
    }
}
impl TableSingletonExt for TArenaRun {
    fn current() -> Self {
        *Self::get_current().unwrap()
    }
    fn get_current() -> Option<Box<Self>> {
        Self::iter().exactly_one().ok().map(|d| Box::new(d))
    }
}
impl TableSingletonExt for TWallet {
    fn current() -> Self {
        *Self::get_current().unwrap()
    }
    fn get_current() -> Option<Box<Self>> {
        Self::iter().exactly_one().ok().map(|d| Box::new(d))
    }
}

pub trait StdbStatusExt {
    fn on_success(&self, f: impl FnOnce(&mut World) + Send + Sync + 'static);
    fn notify_error(&self);
}

impl StdbStatusExt for spacetimedb_sdk::reducer::Status {
    fn on_success(&self, f: impl FnOnce(&mut World) + Send + Sync + 'static) {
        match self {
            StdbStatus::Committed => OperationsPlugin::add(f),
            StdbStatus::Failed(e) => e.notify_error_op(),
            _ => panic!(),
        }
    }
    fn notify_error(&self) {
        match self {
            StdbStatus::Committed => {}
            StdbStatus::Failed(e) => e.notify_error_op(),
            _ => panic!(),
        }
    }
}

pub trait GIDExt {
    fn get_team(self) -> TTeam;
    fn get_user(self) -> TUser;
    fn unit_item(self) -> TUnitItem;
    fn unit_shard_item(self) -> TUnitShardItem;
    fn lootbox_item(self) -> TLootboxItem;
}

impl GIDExt for u64 {
    fn get_team(self) -> TTeam {
        if self == 0 {
            return TTeam {
                id: 0,
                owner: 0,
                units: default(),
                name: default(),
                pool: TeamPool::Owned,
            };
        }
        TTeam::find_by_id(self)
            .with_context(|| format!("Failed to find Team#{self}"))
            .unwrap()
    }
    fn get_user(self) -> TUser {
        if self == 0 {
            return TUser {
                id: 0,
                name: "...".into(),
                identities: default(),
                pass_hash: default(),
                online: default(),
                last_login: default(),
            };
        }
        TUser::find_by_id(self)
            .with_context(|| format!("Failed to find User#{self}"))
            .unwrap()
    }
    fn unit_item(self) -> TUnitItem {
        TUnitItem::find_by_id(self)
            .with_context(|| format!("Failed to find UnitItem#{self}"))
            .unwrap()
    }
    fn unit_shard_item(self) -> TUnitShardItem {
        TUnitShardItem::find_by_id(self)
            .with_context(|| format!("Failed to find UnitShardItem#{self}"))
            .unwrap()
    }
    fn lootbox_item(self) -> TLootboxItem {
        TLootboxItem::find_by_id(self)
            .with_context(|| format!("Failed to find LootboxItem#{self}"))
            .unwrap()
    }
}

pub trait StrExtensions {
    fn split_by_brackets(self, left: char, right: char) -> Vec<(String, bool)>;
    fn extract_bracketed(self, left: char, right: char) -> Vec<String>;
}

impl<'a> StrExtensions for &'a str {
    fn split_by_brackets(mut self, left: char, right: char) -> Vec<(String, bool)> {
        let mut lines: Vec<(String, bool)> = default();
        while let Some(opening) = self.find(left) {
            let left = &self[..opening];
            if let Some(closing) = self.find(right) {
                let mid = &self[opening + 1..closing];
                lines.push((left.to_owned(), false));
                lines.push((mid.to_owned(), true));
                self = &self[closing + 1..];
            } else {
                break;
            }
        }
        lines.push((self.to_owned(), false));
        lines
    }

    fn extract_bracketed(self, left: char, right: char) -> Vec<String> {
        self.split_by_brackets(left, right)
            .into_iter()
            .filter_map(|(s, v)| match v {
                true => Some(s),
                false => None,
            })
            .collect_vec()
    }
}

pub trait BaseUnitExt {
    fn base(&self) -> &str;
    fn base_unit(&self) -> TBaseUnit {
        TBaseUnit::find_by_name(self.base().into()).unwrap()
    }
}

impl BaseUnitExt for FusedUnit {
    fn base(&self) -> &str {
        &self.bases[0]
    }
}
impl BaseUnitExt for String {
    fn base(&self) -> &str {
        self
    }
}
