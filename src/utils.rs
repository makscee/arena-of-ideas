use bevy_egui::{
    egui::{Align2, Context, Id, Pos2},
    EguiContext,
};
use bevy_pkv::GetError;
use bevy_pkv::SetError;

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
    name: &str,
    world: &mut World,
) -> egui::Window<'static> {
    let pos = entity_screen_pos(entity, side, world);
    let side_i = side.as_ivec2();
    let align = match (side_i.x, side_i.y) {
        (-1, 0) => Align2::RIGHT_CENTER,
        (1, 0) => Align2::LEFT_CENTER,
        (0, -1) => Align2::CENTER_TOP,
        (0, 1) => Align2::CENTER_BOTTOM,
        _ => panic!(),
    };

    egui::Window::new(name)
        .id(Id::new(entity).with(name))
        .fixed_pos(Pos2::new(pos.x, pos.y))
        .default_width(10.0)
        .collapsible(false)
        .title_bar(false)
        .resizable(false)
        .pivot(align)
}
pub fn entity_screen_pos(entity: Entity, offset: Vec2, world: &mut World) -> Vec2 {
    let pos = world
        .get::<GlobalTransform>(entity)
        .and_then(|t| Some(t.translation()))
        .unwrap_or_default()
        + vec3(offset.x, offset.y, 0.0);
    world_to_screen(pos, world)
}
pub fn get_insert_t(world: &World) -> f32 {
    world.get_resource::<GameTimer>().unwrap().get_insert_t()
}
pub fn get_t(world: &World) -> f32 {
    world.get_resource::<GameTimer>().unwrap().get_t()
}
pub fn get_parent(entity: Entity, world: &World) -> Entity {
    world.get::<Parent>(entity).unwrap().get()
}
pub fn save_ladder(ladder: &Ladder, world: &mut World) -> Result<(), SetError> {
    world
        .get_resource_mut::<PkvStore>()
        .unwrap()
        .set("ladder", ladder)
}
pub fn load_ladder(world: &World) -> Result<Ladder, GetError> {
    world
        .get_resource::<PkvStore>()
        .unwrap()
        .get::<Ladder>("ladder")
}
pub fn save_active_team(team: &PackedTeam, world: &mut World) -> Result<(), SetError> {
    world
        .get_resource_mut::<PkvStore>()
        .unwrap()
        .set("active_team", team)
}
pub fn load_active_team(world: &World) -> Result<PackedTeam, GetError> {
    world
        .get_resource::<PkvStore>()
        .unwrap()
        .get::<PackedTeam>("active_team")
}
