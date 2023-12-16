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
pub fn egui_context(world: &mut World) -> Context {
    world
        .query::<&mut EguiContext>()
        .single_mut(world)
        .into_inner()
        .get_mut()
        .clone()
}
pub fn world_to_screen(pos: Vec3, world: &mut World) -> Vec2 {
    let (camera, transform) = world.query::<(&Camera, &GlobalTransform)>().single(world);
    camera.world_to_viewport(transform, pos).unwrap_or_default()
}
pub fn screen_to_world(pos: Vec2, camera: &Camera, transform: &GlobalTransform) -> Vec2 {
    camera.viewport_to_world_2d(transform, pos).unwrap()
}
pub fn entity_panel(
    entity: Entity,
    side: Vec2,
    width: Option<f32>,
    name: &str,
    world: &mut World,
) -> egui::Window<'static> {
    let pos = entity_screen_pos(entity, world) + side;
    let side_i = side.as_ivec2();
    let align = match (side_i.x, side_i.y) {
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
pub fn entity_screen_pos(entity: Entity, world: &mut World) -> Vec2 {
    let pos = world
        .get::<GlobalTransform>(entity)
        .map(|t| t.translation())
        .unwrap_or_default();
    world_to_screen(pos, world)
}
pub fn cursor_pos(world: &mut World) -> Option<Vec2> {
    let window = world.query::<&bevy::window::Window>().single(world);
    window.cursor_position()
}
pub fn get_insert_head(world: &World) -> f32 {
    GameTimer::get(world).insert_head()
}
pub fn get_play_head(world: &World) -> f32 {
    GameTimer::get(world).play_head()
}
pub fn start_batch(world: &mut World) {
    GameTimer::get_mut(world).start_batch();
}
pub fn end_batch(world: &mut World) {
    GameTimer::get_mut(world).end_batch();
}
pub fn to_batch_start(world: &mut World) {
    GameTimer::get_mut(world).to_batch_start();
}
pub fn get_parent(entity: Entity, world: &World) -> Entity {
    world.get::<Parent>(entity).unwrap().get()
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

pub trait StrExtensions {
    fn split_by_brackets(self, pattern: (&str, &str)) -> Vec<(String, bool)>;
    fn extract_bracketed(self, pattern: (&str, &str)) -> Vec<String>;
}

impl<'a> StrExtensions for &'a str {
    fn split_by_brackets(mut self, pattern: (&str, &str)) -> Vec<(String, bool)> {
        let mut lines: Vec<(String, bool)> = default();
        while let Some(opening) = self.find(pattern.0) {
            let left = &self[..opening];
            let closing = self.find(pattern.1).unwrap();
            let mid = &self[opening + 1..closing];
            lines.push((left.to_owned(), false));
            lines.push((mid.to_owned(), true));
            self = &self[closing + 1..];
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
