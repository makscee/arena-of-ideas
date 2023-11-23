use std::str::FromStr;

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
    entity_panel(entity, vec2(0.0, 1.0), None, "Ability", world)
        .title_bar(true)
        .show(&ctx, |ui| {
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
    source: &str,
    world: &mut World,
) -> (Vec<(String, Color32)>, Vec<(String, Color32)>) {
    let mut description: Vec<(String, Color32)> = default();
    let mut definitions: Vec<(String, Color32)> = default();

    for (str, extr) in str_extract_brackets(source, ("[", "]")) {
        if extr {
            if let Some(house) = Pools::get_ability_house(&str, world) {
                let color = house.color.clone().into();
                description.push((str.to_owned(), color));
                definitions.push((str, color));
            } else {
                error!("Failed to find house for ability {str}");
            }
        } else {
            description.push((str, Color32::GRAY));
        }
    }

    (description, definitions)
}
pub fn parse_vars(
    source: &str,
    entity: Entity,
    t: f32,
    world: &mut World,
) -> Vec<(String, Color32)> {
    let state = VarState::get(entity, world);
    str_extract_brackets(source, ("{", "}"))
        .into_iter()
        .map(|(str, extr)| match extr {
            true => (
                state
                    .get_string_at(VarName::from_str(&str).unwrap(), t)
                    .unwrap(),
                Color32::GREEN,
            ),
            false => (str, Color32::GRAY),
        })
        .collect_vec()
}
fn str_extract_brackets(mut source: &str, pattern: (&str, &str)) -> Vec<(String, bool)> {
    let mut lines: Vec<(String, bool)> = default();
    while let Some(opening) = source.find(pattern.0) {
        let left = &source[..opening];
        let closing = source.find(pattern.1).unwrap();
        let mid = &source[opening + 1..closing];
        lines.push((left.to_owned(), false));
        lines.push((mid.to_owned(), true));
        source = &source[closing + 1..];
    }
    lines.push((source.to_owned(), false));
    lines
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
pub fn get_insert_head(world: &World) -> f32 {
    GameTimer::get(world).insert_head()
}
pub fn get_play_head(world: &World) -> f32 {
    GameTimer::get(world).play_head()
}
pub fn get_end(world: &World) -> f32 {
    GameTimer::get(world).end()
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
        .set_contents(&text);
    debug!("Saved to clipboard:\n{text}");
}
pub fn get_from_clipboard(world: &mut World) -> Option<String> {
    world
        .resource_mut::<bevy_egui::EguiClipboard>()
        .get_contents()
}
