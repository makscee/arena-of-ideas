use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

// Onboarding tutorial overlay

pub struct OnboardingPlugin;

impl Plugin for OnboardingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OnboardingState>()
            .add_systems(Update, onboarding_ui);
    }
}

#[derive(Resource, Default, PartialEq, Eq)]
pub struct OnboardingState {
    pub show: bool,
    pub step: u8,
}

const STEPS: &[(&str, &str)] = &[
    (
        "Welcome to Arena of Ideas!",
        "An auto-battler where the game's content evolves through player creativity and AI.\n\n\
         Browse the Collection to see available abilities and units.",
    ),
    (
        "Build Your Team",
        "Start a match from the Shop. Buy units, reroll for new offers, and arrange your team.\n\n\
         Units cost gold based on their tier. Sell units you don't need.",
    ),
    (
        "Battle & Fusion",
        "Battles are automatic — your units fight based on their triggers and abilities.\n\n\
         Buy 3 copies of a unit to unlock fusion. Fuse with another unit to create \
         a stronger hybrid with combined abilities!",
    ),
    (
        "Create New Content",
        "Go to Create to breed new abilities from two parents using AI.\n\n\
         Or assemble new units by picking a trigger, abilities, and tier.\n\n\
         Submit your creations to the Incubator for the community to vote on!",
    ),
];

fn onboarding_ui(
    mut contexts: EguiContexts,
    mut state: ResMut<OnboardingState>,
) {
    if !state.show {
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else { return };

    let step = state.step as usize;
    let (title, body) = STEPS.get(step).unwrap_or(&("", ""));

    egui::Window::new("Tutorial")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.set_min_width(400.0);
            ui.heading(*title);
            ui.separator();
            ui.label(*body);
            ui.separator();
            ui.horizontal(|ui| {
                ui.label(format!("Step {} of {}", step + 1, STEPS.len()));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if step + 1 >= STEPS.len() {
                        if ui.button("Got it!").clicked() {
                            state.show = false;
                        }
                    } else {
                        if ui.button("Next →").clicked() {
                            state.step += 1;
                        }
                    }
                    if step > 0 {
                        if ui.button("← Back").clicked() {
                            state.step -= 1;
                        }
                    }
                    if ui.button("Skip").clicked() {
                        state.show = false;
                    }
                });
            });
        });
}
