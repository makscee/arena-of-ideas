use egui::{epaint, Label, Rect};

use super::*;

pub struct TextColumnPlugin;

#[derive(Component)]
pub struct TextColumn {
    lines: Vec<(f32, Cstr)>,
    entity: Entity,
}

impl Plugin for TextColumnPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, Self::ui);
    }
}

impl TextColumnPlugin {
    fn ui(world: &mut World) {
        let ctx = &if let Some(context) = egui_context(world) {
            context
        } else {
            return;
        };
        let mut drawn: Vec<Vec<Rect>> = [default()].into();
        let t = GameTimer::get().play_head();
        let start_height = world_to_screen(vec3(0.0, 2.0, 0.0), world).y;
        const Y_PER_LEVEL: f32 = 20.0;
        CentralPanel::default()
            .frame(Frame::none())
            .show(ctx, |ui| {
                for entity in UnitPlugin::collect_all(world) {
                    let state = VarState::get(entity, world);
                    let text = state.get_string_at(VarName::Name, t).unwrap();
                    let pos = state
                        .get_value_at(VarName::Position, t)
                        .unwrap()
                        .get_vec2()
                        .unwrap()
                        .extend(0.0);
                    let x = world_to_screen(pos, world).x;
                    let (_, galley, response) =
                        Label::new(RichText::new(text).color(WHITE)).layout_in_ui(ui);
                    let rect = galley.rect;
                    let rect = rect.translate(egui::vec2(x - rect.width() * 0.5, 0.0));
                    let mut cur_lvl = 0;
                    while drawn[cur_lvl].iter().any(|r| r.intersects(rect)) {
                        cur_lvl += 1;
                        if drawn.get(cur_lvl).is_none() {
                            drawn.push(default());
                        }
                    }
                    drawn[cur_lvl].push(rect);
                    let rect = rect
                        .translate(egui::vec2(0.0, start_height - cur_lvl as f32 * Y_PER_LEVEL));
                    ui.painter()
                        .add(epaint::TextShape::new(rect.left_top(), galley, LIGHT_GRAY));
                }
            });
    }
}
