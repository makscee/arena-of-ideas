use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use crate::resources::game_state::GameState;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .add_systems(
                bevy_egui::EguiPrimaryContextPass,
                title_ui.run_if(in_state(GameState::Title)),
            )
            .add_systems(
                bevy_egui::EguiPrimaryContextPass,
                login_ui.run_if(in_state(GameState::Login)),
            );
    }
}

fn title_ui(mut contexts: EguiContexts, mut next_state: ResMut<NextState<GameState>>) {
    let Ok(ctx) = contexts.ctx_mut() else { return };

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(ui.available_height() * 0.3);
            ui.heading(
                egui::RichText::new("Arena of Ideas")
                    .size(48.0)
                    .color(egui::Color32::WHITE),
            );
            ui.add_space(20.0);
            if ui
                .button(egui::RichText::new("Start Game").size(24.0))
                .clicked()
            {
                next_state.set(GameState::Login);
            }
            ui.add_space(10.0);
            ui.colored_label(egui::Color32::from_rgb(150, 150, 150), "or press SPACE");
        });
    });

    // Also handle keyboard
    if ctx.input(|i| i.key_pressed(egui::Key::Space)) {
        next_state.set(GameState::Login);
    }
}

fn login_ui(mut contexts: EguiContexts) {
    let Ok(ctx) = contexts.ctx_mut() else { return };

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(ui.available_height() * 0.4);
            ui.heading(
                egui::RichText::new("Connecting...")
                    .size(32.0)
                    .color(egui::Color32::WHITE),
            );
            ui.add_space(10.0);
            ui.spinner();
        });
    });
}
