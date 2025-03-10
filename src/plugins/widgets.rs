use bevy::{
    ecs::schedule::Condition,
    input::common_conditions::{input_just_pressed, input_pressed},
};
use egui::Area;

use super::*;

pub struct WidgetsPlugin;

impl Plugin for WidgetsPlugin {
    fn build(&self, app: &mut App) {
        // app.add_systems(Update, Self::ui);

        // if cfg!(debug_assertions) {
        //     app.add_systems(
        //         Update,
        //         give_c.run_if(
        //             input_just_pressed(KeyCode::KeyG)
        //                 .and_then(in_state(GameState::Title).or_else(in_state(GameState::Meta))),
        //         ),
        //     )
        //     .add_systems(
        //         Update,
        //         cmd_test.run_if(
        //             input_pressed(KeyCode::SuperLeft).and_then(input_pressed(KeyCode::ShiftLeft)),
        //         ),
        //     );
        // }
    }
}

fn cmd_test(world: &mut World) {
    let content = |ui: &mut Ui, _: &mut World| {
        "12345678910 11 12 13 14 15 16 17 18 19 20".cstr().label(ui);
        // br(ui);
        "test test test test test test test test test"
            .cstr()
            .label(ui);
        space(ui);
        "test test test test test test test test test"
            .cstr()
            .label(ui);
    };
    if just_pressed(KeyCode::KeyA, world) {
        Tile::new(Side::Left, content).push(world);
    }
    if just_pressed(KeyCode::KeyD, world) {
        Tile::new(Side::Right, content).push(world);
    }
    if just_pressed(KeyCode::KeyW, world) {
        Tile::new(Side::Top, content).push(world);
    }
    if just_pressed(KeyCode::KeyS, world) {
        Tile::new(Side::Bottom, content).push(world);
    }
    if just_pressed(KeyCode::KeyN, world) {
        "test test test test test test test test test test test test test test test".notify(world);
    }
    if just_pressed(KeyCode::KeyC, world) {
        Confirmation::new("Test")
            .accept(|_| {})
            .cancel(|_| {})
            .push(world);
    }
}

impl WidgetsPlugin {
    fn ui(world: &mut World) {
        let Some(ctx) = &egui_context(world) else {
            return;
        };

        let state = cur_state(world);

        TilePlugin::show_all(ctx, world);
        WindowPlugin::show_all(ctx, world);

        // Content
        CentralPanel::default()
            .frame(Frame::new())
            .show(ctx, |ui| match state {
                GameState::Connect => ConnectPlugin::tab(ui),
                GameState::Login => LoginPlugin::tab_login(ui, world),
                _ => {}
            });

        // Overlay
        Confirmation::show_current(ctx, world);
        Notification::show_recent(ctx, world);

        Area::new(Id::new("corner_info"))
            .anchor(Align2::RIGHT_BOTTOM, [0.0, -20.0])
            .order(Order::Tooltip)
            .show(ctx, |ui| {
                let ui = &mut ui.child_ui(
                    ui.available_rect_before_wrap(),
                    Layout::right_to_left(Align::Min),
                    None,
                );
                ui.add_space(13.0);
                if let Some(fps) = world
                    .resource::<DiagnosticsStore>()
                    .get(&FrameTimeDiagnosticsPlugin::FPS)
                {
                    if let Some(fps) = fps.smoothed() {
                        format!("[vd fps:] {fps:.0}").label(ui);
                    }
                }
                VERSION.cstr().label(ui);
                current_server()
                    .1
                    .cstr_cs(VISIBLE_DARK, CstrStyle::Bold)
                    .label(ui);
            });
    }
}
