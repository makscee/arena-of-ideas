use bevy::input::mouse::MouseButton;
use spacetimedb_sdk::table::TableType;

use super::*;

pub fn just_pressed(key: KeyCode, world: &World) -> bool {
    world.resource::<ButtonInput<KeyCode>>().just_pressed(key)
}
pub fn left_mouse_pressed(world: &World) -> bool {
    world
        .resource::<ButtonInput<MouseButton>>()
        .pressed(MouseButton::Left)
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
pub fn elapsed_time(world: &World) -> f32 {
    world.resource::<Time>().elapsed_seconds()
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
// pub fn get_context_expression(world: &mut World, key: &str) -> Expression {
//     let id = Id::new(key);
//     if let Some(context) = egui_context(world) {
//         context.data(|r| r.get_temp::<Expression>(id).unwrap_or_default())
//     } else {
//         default()
//     }
// }
// pub fn set_context_expression(world: &mut World, key: &str, value: Expression) {
//     let id = Id::new(key);
//     if let Some(context) = egui_context(world) {
//         context.data_mut(|w| w.insert_temp(id, value))
//     }
// }
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
// pub fn entity_window(
//     entity: Entity,
//     side: Vec2,
//     width: Option<f32>,
//     name: &str,
//     world: &World,
// ) -> egui::Window<'static> {
//     let pos = entity_screen_pos(entity, side, world);
//     let side_i = side.as_ivec2();
//     let align = match (side_i.x.signum(), side_i.y.signum()) {
//         (-1, 0) => Align2::RIGHT_CENTER,
//         (1, 0) => Align2::LEFT_CENTER,
//         (0, -1) => Align2::CENTER_TOP,
//         (0, 1) => Align2::CENTER_BOTTOM,
//         (0, 0) => Align2::CENTER_CENTER,
//         _ => panic!(),
//     };

//     egui::Window::new(name)
//         .id(Id::new(entity).with(name))
//         .fixed_pos(Pos2::new(pos.x, pos.y))
//         .default_width(width.unwrap_or(100.0))
//         .collapsible(false)
//         .title_bar(false)
//         .resizable(false)
//         .pivot(align)
// }
// pub fn entity_screen_pos(entity: Entity, offset: Vec2, world: &World) -> Vec2 {
//     let pos = world
//         .get::<GlobalTransform>(entity)
//         .map(|t| t.translation())
//         .unwrap_or_default();
//     world_to_screen(pos + offset.extend(0.0), world)
// }
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
    GlobalSettings::filter_by_always_zero(0).unwrap()
}
pub fn app_exit(world: &mut World) {
    world
        .get_resource_mut::<bevy::prelude::Events<bevy::app::AppExit>>()
        .unwrap()
        .send(bevy::app::AppExit);
}
pub fn cur_state(world: &World) -> GameState {
    *world.resource::<State<GameState>>().get()
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
            .get_var(VarName::Faction, world)
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
        Color::rgba_u8(a[0], a[1], a[2], a[3])
    }
}

pub trait ToC32 {
    fn c32(&self) -> Color32;
}

impl ToC32 for Color {
    fn c32(&self) -> Color32 {
        let c = self.as_rgba_u8();
        Color32::from_rgba_unmultiplied(c[0], c[1], c[2], c[3])
    }
}

pub trait WorldExt {
    fn game_clear(&mut self);
}

impl WorldExt for World {
    fn game_clear(&mut self) {
        Representation::despawn_all(self);
    }
}

pub trait TableExt {
    fn current() -> Self;
    fn get_current() -> Option<Box<Self>>;
}

impl TableExt for TArenaRun {
    fn current() -> Self {
        *Self::get_current().unwrap()
    }
    fn get_current() -> Option<Box<Self>> {
        TArenaRun::iter().exactly_one().ok().map(|d| Box::new(d))
    }
}
