use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use shared::battle::{BattleAction, BattleResult, BattleSide};

use crate::plugins::ui::colors;
use crate::resources::game_state::GameState;

pub struct BattleViewerPlugin;

impl Plugin for BattleViewerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BattleViewerState>()
            // Disabled — superseded by BattleScenePlugin
            ;
    }
}

#[derive(Resource, Default)]
pub struct BattleViewerState {
    /// The battle result to display. Set by the match system before entering Battle state.
    pub result: Option<BattleResult>,
    /// Current action index for replay playback.
    pub action_index: usize,
    /// Whether auto-playing.
    pub playing: bool,
    /// Timer for auto-play.
    pub timer: f32,
    /// Speed of auto-play (seconds per action).
    pub speed: f32,
}

impl BattleViewerState {
    pub fn load_battle(&mut self, result: BattleResult) {
        self.result = Some(result);
        self.action_index = 0;
        self.playing = false;
        self.timer = 0.0;
        self.speed = 0.3;
    }
}

fn battle_viewer_ui(
    mut contexts: EguiContexts,
    mut state: ResMut<BattleViewerState>,
    time: Res<Time>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };

    if state.result.is_none() {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("No battle to display");
            if ui.button("Back to Shop").clicked() {
                next_state.set(GameState::Shop);
            }
        });
        return;
    };

    let total_actions = state.result.as_ref().unwrap().actions.len();
    let current_idx = state.action_index;

    // Auto-play
    if state.playing && current_idx < total_actions {
        state.timer += time.delta_secs();
        if state.timer >= state.speed {
            state.timer = 0.0;
            state.action_index += 1;
        }
    }

    let result = state.result.clone().unwrap();

    egui::TopBottomPanel::top("battle_top").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.heading("Battle Replay");
            ui.separator();

            let winner_text = match result.winner {
                BattleSide::Left => "Left Team Wins!",
                BattleSide::Right => "Right Team Wins!",
            };
            let winner_color = match result.winner {
                BattleSide::Left => colors::RATING_POSITIVE,
                BattleSide::Right => colors::RATING_NEGATIVE,
            };
            ui.colored_label(winner_color, winner_text);
            ui.label(format!("({} turns)", result.turns));
        });
    });

    egui::TopBottomPanel::bottom("battle_controls").show(ctx, |ui| {
        ui.horizontal(|ui| {
            if ui.button("⏮").clicked() {
                state.action_index = 0;
            }
            if ui.button("⏪").clicked() {
                state.action_index = state.action_index.saturating_sub(1);
            }
            if state.playing {
                if ui.button("⏸").clicked() {
                    state.playing = false;
                }
            } else if ui.button("▶").clicked() {
                state.playing = true;
            }
            if ui.button("⏩").clicked() {
                state.action_index = (state.action_index + 1).min(total_actions);
            }
            if ui.button("⏭").clicked() {
                state.action_index = total_actions;
            }
            ui.separator();
            ui.label(format!("{} / {}", current_idx, total_actions));
            ui.separator();
            ui.label("Speed:");
            ui.add(egui::Slider::new(&mut state.speed, 0.05..=1.0).suffix("s"));
            ui.separator();
            if ui.button("Back to Shop").clicked() {
                state.result = None;
                next_state.set(GameState::Shop);
            }
        });
    });

    egui::CentralPanel::default().show(ctx, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            for (i, action) in result.actions.iter().enumerate() {
                if i >= current_idx {
                    break;
                }
                let text = format_action(action);
                let color = action_color(action);
                ui.colored_label(color, format!("[{}] {}", i + 1, text));
            }
        });
    });
}

fn format_action(action: &BattleAction) -> String {
    match action {
        BattleAction::Spawn { unit, slot, side } => {
            format!("Unit {} spawns at slot {} ({:?})", unit, slot, side)
        }
        BattleAction::Damage {
            source,
            target,
            amount,
        } => {
            format!("Unit {} deals {} damage to Unit {}", source, amount, target)
        }
        BattleAction::Heal {
            source,
            target,
            amount,
        } => {
            format!("Unit {} heals Unit {} for {}", source, target, amount)
        }
        BattleAction::Death { unit } => {
            format!("Unit {} dies!", unit)
        }
        BattleAction::StatChange { unit, stat, delta } => {
            format!("Unit {} {:?} {:+}", unit, stat, delta)
        }
        BattleAction::Vfx { unit, effect } => {
            format!("VFX: {} on Unit {}", effect, unit)
        }
        BattleAction::Wait { seconds } => {
            format!("Wait {:.1}s", seconds)
        }
        BattleAction::Fatigue { amount } => {
            format!("Fatigue! All units take {} damage", amount)
        }
        BattleAction::AbilityUsed {
            source,
            ability_name,
        } => {
            format!("Unit {} uses {}", source, ability_name)
        }
    }
}

fn action_color(action: &BattleAction) -> egui::Color32 {
    match action {
        BattleAction::Damage { .. } => egui::Color32::from_rgb(255, 100, 100),
        BattleAction::Heal { .. } => egui::Color32::from_rgb(100, 255, 100),
        BattleAction::Death { .. } => egui::Color32::from_rgb(255, 50, 50),
        BattleAction::AbilityUsed { .. } => egui::Color32::from_rgb(100, 180, 255),
        BattleAction::Fatigue { .. } => egui::Color32::from_rgb(255, 200, 50),
        BattleAction::Spawn { .. } => egui::Color32::from_rgb(200, 200, 200),
        _ => egui::Color32::GRAY,
    }
}
