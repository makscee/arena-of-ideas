use bevy_egui::{
    egui::{Align2, Context, Id, Pos2},
    EguiContext,
};

use super::*;

pub fn just_pressed(key: KeyCode, world: &World) -> bool {
    world
        .get_resource::<Input<KeyCode>>()
        .unwrap()
        .just_pressed(key)
}
pub fn egui_context(world: &mut World) -> Option<Context> {
    world
        .query::<&mut EguiContext>()
        .get_single_mut(world)
        .map(|c| c.into_inner().get_mut().clone())
        .ok()
}
pub fn get_context_bool(world: &mut World, key: &str) -> bool {
    let id = Id::new(key);
    if let Some(context) = egui_context(world) {
        context.data(|r| r.get_temp::<bool>(id).unwrap_or_default())
    } else {
        default()
    }
}
pub fn set_context_bool(world: &mut World, key: &str, value: bool) {
    let id = Id::new(key);
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
pub fn get_context_expression(world: &mut World, key: &str) -> Expression {
    let id = Id::new(key);
    if let Some(context) = egui_context(world) {
        context.data(|r| r.get_temp::<Expression>(id).unwrap_or_default())
    } else {
        default()
    }
}
pub fn set_context_expression(world: &mut World, key: &str, value: Expression) {
    let id = Id::new(key);
    if let Some(context) = egui_context(world) {
        context.data_mut(|w| w.insert_temp(id, value))
    }
}
pub fn world_to_screen(pos: Vec3, world: &World) -> Vec2 {
    let entity = world.entity(world.resource::<CameraData>().entity);
    let camera = entity.get::<Camera>().unwrap();
    let transform = entity.get::<GlobalTransform>().unwrap();
    camera.world_to_viewport(transform, pos).unwrap_or_default()
}
pub fn screen_to_world(pos: Vec2, camera: &Camera, transform: &GlobalTransform) -> Vec2 {
    camera.viewport_to_world_2d(transform, pos).unwrap()
}
pub fn entity_window(
    entity: Entity,
    side: Vec2,
    width: Option<f32>,
    name: &str,
    world: &World,
) -> egui::Window<'static> {
    let pos = entity_screen_pos(entity, side, world);
    let side_i = side.as_ivec2();
    let align = match (side_i.x.signum(), side_i.y.signum()) {
        (-1, 0) => Align2::RIGHT_CENTER,
        (1, 0) => Align2::LEFT_CENTER,
        (0, -1) => Align2::CENTER_TOP,
        (0, 1) => Align2::CENTER_BOTTOM,
        (0, 0) => Align2::CENTER_CENTER,
        _ => panic!(),
    };

    egui::Window::new(name)
        .id(Id::new(entity).with(name))
        .fixed_pos(Pos2::new(pos.x, pos.y))
        .default_width(width.unwrap_or(100.0))
        .collapsible(false)
        .title_bar(false)
        .resizable(false)
        .pivot(align)
}
pub fn entity_screen_pos(entity: Entity, offset: Vec2, world: &World) -> Vec2 {
    let pos = world
        .get::<GlobalTransform>(entity)
        .map(|t| t.translation())
        .unwrap_or_default();
    world_to_screen(pos + offset.extend(0.0), world)
}
pub fn cursor_pos(world: &mut World) -> Option<Vec2> {
    let window = world.query::<&bevy::window::Window>().single(world);
    window.cursor_position()
}
pub fn get_children(entity: Entity, world: &World) -> Vec<Entity> {
    world.get::<Children>(entity).unwrap().to_vec()
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
pub fn draw_curve(p1: Pos2, p2: Pos2, p3: Pos2, p4: Pos2, width: f32, color: Color32, ui: &mut Ui) {
    let points = [p1, p2, p3, p4];
    let curve = egui::Shape::CubicBezier(egui::epaint::CubicBezierShape::from_points_stroke(
        points,
        false,
        Color32::TRANSPARENT,
        Stroke { width, color },
    ));
    ui.painter().add(curve);
}

pub trait StrExtensions {
    fn split_by_brackets(self, pattern: (&str, &str)) -> Vec<(String, bool)>;
    fn extract_bracketed(self, pattern: (&str, &str)) -> Vec<String>;
}

impl<'a> StrExtensions for &'a str {
    fn split_by_brackets(mut self, pattern: (&str, &str)) -> Vec<(String, bool)> {
        let mut lines: Vec<(String, bool)> = default();
        while let Some(opening) = self.find(pattern.0) {
            let left = &self[..opening];
            if let Some(closing) = self.find(pattern.1) {
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

    fn extract_bracketed(self, pattern: (&str, &str)) -> Vec<String> {
        self.split_by_brackets(pattern)
            .into_iter()
            .filter_map(|(s, v)| match v {
                true => Some(s),
                false => None,
            })
            .collect_vec()
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
}

impl ToPos2 for Vec2 {
    fn to_pos2(&self) -> Pos2 {
        pos2(self.x, self.y)
    }
}

pub trait EntityExt {
    fn get_parent(&self, world: &World) -> Option<Entity>;
    fn get_parent_query(&self, query: &Query<&Parent>) -> Option<Entity>;
}

impl EntityExt for Entity {
    fn get_parent(&self, world: &World) -> Option<Entity> {
        world.get::<Parent>(*self).map(|p| p.get())
    }

    fn get_parent_query(&self, query: &Query<&Parent>) -> Option<Entity> {
        query.get(*self).ok().map(|p| p.get())
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
