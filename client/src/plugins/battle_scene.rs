use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use shared::battle::{BattleAction, BattleResult, BattleSide};

use crate::plugins::painter::paint_default_unit;
use crate::resources::game_state::GameState;

pub struct BattleScenePlugin;

impl Plugin for BattleScenePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BattleSceneState>()
            .init_resource::<BattleFrameCount>()
            .add_systems(OnEnter(GameState::Battle), reset_frame_count)
            .add_systems(Update, battle_scene_ui.run_if(in_state(GameState::Battle)));
    }
}

#[derive(Resource, Default)]
struct BattleFrameCount(u32);

fn reset_frame_count(mut count: ResMut<BattleFrameCount>) {
    count.0 = 0;
}

/// A unit's visual state during battle replay.
#[derive(Clone, Debug)]
pub struct BattleUnitVisual {
    pub id: u64,
    pub name: String,
    pub hp: i32,
    pub pwr: i32,
    pub dmg: i32,
    pub alive: bool,
    pub side: BattleSide,
    pub slot: u8,
    pub color: egui::Color32,
}

/// State for the battle scene visualization.
#[derive(Resource, Default)]
pub struct BattleSceneState {
    pub result: Option<BattleResult>,
    pub units: Vec<BattleUnitVisual>,
    pub action_index: usize,
    pub playing: bool,
    pub timer: f32,
    pub speed: f32,
    pub log: Vec<String>,
}

impl BattleSceneState {
    pub fn load(&mut self, result: BattleResult, units: Vec<BattleUnitVisual>) {
        self.result = Some(result);
        self.units = units;
        self.action_index = 0;
        self.playing = false;
        self.timer = 0.0;
        self.speed = 0.5;
        self.log.clear();
    }
}

/// Apply battle actions up to the current index, mutating unit visuals.
fn apply_actions_to(state: &mut BattleSceneState) {
    // Reset units to initial state
    for u in &mut state.units {
        u.dmg = 0;
        u.alive = true;
    }
    state.log.clear();

    let Some(ref result) = state.result else {
        return;
    };

    for (i, action) in result.actions.iter().enumerate() {
        if i >= state.action_index {
            break;
        }
        match action {
            BattleAction::Damage {
                source,
                target,
                amount,
            } => {
                if let Some(u) = state.units.iter_mut().find(|u| u.id == *target) {
                    u.dmg += amount;
                }
                state
                    .log
                    .push(format!("{} deals {} dmg to {}", source, amount, target));
            }
            BattleAction::Heal {
                source,
                target,
                amount,
            } => {
                if let Some(u) = state.units.iter_mut().find(|u| u.id == *target) {
                    u.dmg = (u.dmg - amount).max(0);
                }
                state
                    .log
                    .push(format!("{} heals {} for {}", source, target, amount));
            }
            BattleAction::Death { unit } => {
                if let Some(u) = state.units.iter_mut().find(|u| u.id == *unit) {
                    u.alive = false;
                }
                state.log.push(format!("Unit {} dies!", unit));
            }
            BattleAction::StatChange { unit, stat, delta } => {
                if let Some(u) = state.units.iter_mut().find(|u| u.id == *unit) {
                    match stat {
                        shared::battle::StatKind::Pwr => u.pwr = (u.pwr + delta).max(0),
                        shared::battle::StatKind::Hp => u.hp = (u.hp + delta).max(0),
                        _ => {}
                    }
                }
                state
                    .log
                    .push(format!("Unit {} {:?} {:+}", unit, stat, delta));
            }
            BattleAction::AbilityUsed {
                source,
                ability_name,
            } => {
                state
                    .log
                    .push(format!("Unit {} uses {}", source, ability_name));
            }
            BattleAction::Fatigue { amount } => {
                for u in state.units.iter_mut().filter(|u| u.alive) {
                    u.dmg += amount;
                }
                state.log.push(format!("Fatigue! All take {} dmg", amount));
            }
            _ => {}
        }
    }
}

fn battle_scene_ui(
    mut contexts: EguiContexts,
    mut state: ResMut<BattleSceneState>,
    time: Res<Time>,
    mut next_state: ResMut<NextState<GameState>>,
    mut frame_count: ResMut<BattleFrameCount>,
) {
    // Skip first 2 frames to let egui initialize
    frame_count.0 += 1;
    if frame_count.0 < 3 {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };

    if state.result.is_none() {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("No battle loaded");
            if ui.button("Back").clicked() {
                next_state.set(GameState::Home);
            }
        });
        return;
    }

    let total_actions = state.result.as_ref().unwrap().actions.len();

    // Auto-play
    if state.playing && state.action_index < total_actions {
        state.timer += time.delta_secs();
        if state.timer >= state.speed {
            state.timer = 0.0;
            state.action_index += 1;
            apply_actions_to(&mut state);
        }
    }

    let winner = state.result.as_ref().unwrap().winner;
    let turns = state.result.as_ref().unwrap().turns;

    // Top bar
    egui::TopBottomPanel::top("battle_scene_top").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.heading("Battle");
            ui.separator();
            let (text, color) = match winner {
                BattleSide::Left => ("Left Wins!", egui::Color32::from_rgb(100, 255, 100)),
                BattleSide::Right => ("Right Wins!", egui::Color32::from_rgb(255, 100, 100)),
            };
            ui.colored_label(color, text);
            ui.label(format!("{} turns", turns));
        });
    });

    // Controls
    egui::TopBottomPanel::bottom("battle_scene_controls").show(ctx, |ui| {
        ui.horizontal(|ui| {
            if ui.button("⏮").clicked() {
                state.action_index = 0;
                apply_actions_to(&mut state);
            }
            if ui.button("⏪").clicked() {
                state.action_index = state.action_index.saturating_sub(1);
                apply_actions_to(&mut state);
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
                apply_actions_to(&mut state);
            }
            if ui.button("⏭").clicked() {
                state.action_index = total_actions;
                apply_actions_to(&mut state);
            }
            ui.separator();
            ui.label(format!("{} / {}", state.action_index, total_actions));
            ui.add(egui::Slider::new(&mut state.speed, 0.05..=2.0).text("speed"));
            ui.separator();
            if ui.button("Back").clicked() {
                next_state.set(GameState::Home);
            }
        });
    });

    // Battle area
    egui::CentralPanel::default().show(ctx, |ui| {
        let available = ui.available_rect_before_wrap();
        let battle_height = available.height() * 0.6;
        let log_height = available.height() * 0.4;

        // Unit rendering area
        let battle_rect =
            egui::Rect::from_min_size(available.min, egui::vec2(available.width(), battle_height));

        // Draw battlefield background
        ui.painter()
            .rect_filled(battle_rect, 0.0, egui::Color32::from_rgb(20, 20, 30));

        // Draw units
        let unit_size = (battle_rect.height() * 0.35).min(80.0);
        let left_units: Vec<_> = state
            .units
            .iter()
            .filter(|u| u.side == BattleSide::Left)
            .collect();
        let right_units: Vec<_> = state
            .units
            .iter()
            .filter(|u| u.side == BattleSide::Right)
            .collect();

        let left_x = battle_rect.left() + battle_rect.width() * 0.25;
        let right_x = battle_rect.left() + battle_rect.width() * 0.75;

        for (i, unit) in left_units.iter().enumerate() {
            let y = battle_rect.top()
                + (i as f32 + 0.5) * battle_rect.height() / left_units.len().max(1) as f32;
            let rect = egui::Rect::from_center_size(
                egui::pos2(left_x, y),
                egui::vec2(unit_size, unit_size),
            );
            let alpha = if unit.alive { 255 } else { 60 };
            let color = egui::Color32::from_rgba_premultiplied(
                unit.color.r(),
                unit.color.g(),
                unit.color.b(),
                alpha,
            );
            paint_default_unit(rect, color, unit.hp, unit.pwr, unit.dmg, &unit.name, ui);
        }

        for (i, unit) in right_units.iter().enumerate() {
            let y = battle_rect.top()
                + (i as f32 + 0.5) * battle_rect.height() / right_units.len().max(1) as f32;
            let rect = egui::Rect::from_center_size(
                egui::pos2(right_x, y),
                egui::vec2(unit_size, unit_size),
            );
            let alpha = if unit.alive { 255 } else { 60 };
            let color = egui::Color32::from_rgba_premultiplied(
                unit.color.r(),
                unit.color.g(),
                unit.color.b(),
                alpha,
            );
            paint_default_unit(rect, color, unit.hp, unit.pwr, unit.dmg, &unit.name, ui);
        }

        // VS label
        ui.painter().text(
            battle_rect.center(),
            egui::Align2::CENTER_CENTER,
            "VS",
            egui::FontId::proportional(24.0),
            egui::Color32::from_rgb(150, 150, 150),
        );

        // Battle log
        ui.separator();
        egui::ScrollArea::vertical()
            .max_height(log_height)
            .stick_to_bottom(true)
            .show(ui, |ui| {
                for (i, entry) in state.log.iter().enumerate() {
                    let color = if entry.contains("dies") {
                        egui::Color32::from_rgb(255, 50, 50)
                    } else if entry.contains("heals") {
                        egui::Color32::from_rgb(100, 255, 100)
                    } else if entry.contains("uses") {
                        egui::Color32::from_rgb(100, 180, 255)
                    } else if entry.contains("Fatigue") {
                        egui::Color32::from_rgb(255, 200, 50)
                    } else {
                        egui::Color32::GRAY
                    };
                    ui.colored_label(color, format!("[{}] {}", i + 1, entry));
                }
            });
    });
}
