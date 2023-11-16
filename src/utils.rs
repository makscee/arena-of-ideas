use bevy_egui::{
    egui::{Align2, Context, Id, Pos2},
    EguiContext,
};
use ecolor::Color32;

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
        .default_width(width.unwrap_or(100.0))
        .collapsible(false)
        .title_bar(false)
        .resizable(false)
        .pivot(align)
}
pub fn show_description_panels(entity: Entity, description: &str, world: &mut World) {
    let (description, definitions) = parse_description(description, world);
    let ctx = egui_context(world);
    entity_panel(entity, vec2(0.0, 1.0), None, "description", world).show(&ctx, |ui| {
        let mut job = LayoutJob::default();
        for (text, color) in description {
            job.append(
                &text,
                0.0,
                TextFormat {
                    font_id: FontId::new(14.0, FontFamily::Proportional),
                    color,
                    ..Default::default()
                },
            );
        }
        ui.label(WidgetText::LayoutJob(job));
    });
    if !definitions.is_empty() && world.resource::<HoveredUnit>().0 == Some(entity) {
        entity_panel(entity, vec2(-1.0, 0.0), Some(200.0), "Definitions", world)
            .title_bar(true)
            .show(&ctx, |ui| {
                for (name, color) in definitions {
                    let description = &Pools::get_ability(&name, world).description;
                    ui.heading(RichText::new(name).color(color).strong());
                    ui.label(description);
                }
            });
    }
}
pub fn parse_description(
    mut source: &str,
    world: &mut World,
) -> (Vec<(String, Color32)>, Vec<(String, Color32)>) {
    let mut description: Vec<(String, Color32)> = default();
    let mut definitions: Vec<(String, Color32)> = default();
    while let Some(pos) = source.find("[") {
        let left = &source[..pos];
        let pos2 = source.find("]").unwrap();
        let mid = &source[pos + 1..pos2];
        description.push((left.to_owned(), Color32::WHITE));
        let color = Pools::get_ability_house(mid, world).color.clone().into();
        description.push((mid.to_owned(), color));
        definitions.push((mid.to_owned(), color));
        source = &source[pos2 + 1..];
    }
    description.push((source.to_owned(), Color32::WHITE));

    (description, definitions)
}
pub fn entity_screen_pos(entity: Entity, offset: Vec2, world: &mut World) -> Vec2 {
    let pos = world
        .get::<GlobalTransform>(entity)
        .and_then(|t| Some(t.translation()))
        .unwrap_or_default()
        + vec3(offset.x, offset.y, 0.0);
    world_to_screen(pos, world)
}
pub fn cursor_pos(world: &mut World) -> Option<Vec2> {
    let window = world.query::<&bevy::window::Window>().single(world);
    window.cursor_position()
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
