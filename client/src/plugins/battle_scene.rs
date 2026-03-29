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
            .add_systems(
                bevy_egui::EguiPrimaryContextPass,
                battle_scene_ui.run_if(in_state(GameState::Battle)),
            );
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
    pub initial_hp: i32,
    pub initial_pwr: i32,
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
    pub initial_units: Vec<BattleUnitVisual>,
    pub units: Vec<BattleUnitVisual>,
    pub action_index: usize,
    pub playing: bool,
    pub timer: f32,
    pub speed: f32,
    pub log: Vec<(String, egui::Color32)>,
}

impl BattleSceneState {
    pub fn load(&mut self, result: BattleResult, units: Vec<BattleUnitVisual>) {
        self.initial_units = units.clone();
        self.units = units;
        self.result = Some(result);
        self.action_index = 0;
        self.playing = false;
        self.timer = 0.0;
        self.speed = 0.3;
        self.log.clear();
    }

    /// Reset units to initial state and replay actions up to current index.
    fn rebuild_state(&mut self) {
        self.units = self.initial_units.clone();
        self.log.clear();

        let Some(ref result) = self.result else {
            return;
        };

        for i in 0..self.action_index.min(result.actions.len()) {
            let action = &result.actions[i];
            let (text, color) = match action {
                BattleAction::Damage {
                    source,
                    target,
                    amount,
                } => {
                    if let Some(u) = self.units.iter_mut().find(|u| u.id == *target) {
                        u.dmg += amount;
                        if u.hp - u.dmg <= 0 {
                            u.alive = false;
                        }
                    }
                    (
                        format!("#{} → {} dmg → #{}", source, amount, target),
                        egui::Color32::from_rgb(255, 120, 120),
                    )
                }
                BattleAction::Heal {
                    source,
                    target,
                    amount,
                } => {
                    if let Some(u) = self.units.iter_mut().find(|u| u.id == *target) {
                        u.dmg = (u.dmg - amount).max(0);
                    }
                    (
                        format!("#{} heals #{} for {}", source, target, amount),
                        egui::Color32::from_rgb(120, 255, 120),
                    )
                }
                BattleAction::Death { unit } => {
                    if let Some(u) = self.units.iter_mut().find(|u| u.id == *unit) {
                        u.alive = false;
                    }
                    let name = self
                        .units
                        .iter()
                        .find(|u| u.id == *unit)
                        .map(|u| u.name.as_str())
                        .unwrap_or("?");
                    (
                        format!("{} dies!", name),
                        egui::Color32::from_rgb(255, 50, 50),
                    )
                }
                BattleAction::StatChange { unit, stat, delta } => {
                    if let Some(u) = self.units.iter_mut().find(|u| u.id == *unit) {
                        match stat {
                            shared::battle::StatKind::Pwr => u.pwr = (u.pwr + delta).max(0),
                            shared::battle::StatKind::Hp => u.hp = (u.hp + delta).max(0),
                            _ => {}
                        }
                    }
                    (
                        format!("#{} {:?} {:+}", unit, stat, delta),
                        egui::Color32::from_rgb(200, 200, 100),
                    )
                }
                BattleAction::AbilityUsed {
                    source,
                    ability_name,
                } => {
                    let name = self
                        .units
                        .iter()
                        .find(|u| u.id == *source)
                        .map(|u| u.name.as_str())
                        .unwrap_or("?");
                    (
                        format!("{} uses {}", name, ability_name),
                        egui::Color32::from_rgb(120, 180, 255),
                    )
                }
                BattleAction::Fatigue { amount } => {
                    for u in self.units.iter_mut().filter(|u| u.alive) {
                        u.dmg += amount;
                    }
                    (
                        format!("FATIGUE! All take {} dmg", amount),
                        egui::Color32::from_rgb(255, 200, 50),
                    )
                }
                BattleAction::Spawn { unit, side, .. } => {
                    let name = self
                        .units
                        .iter()
                        .find(|u| u.id == *unit)
                        .map(|u| u.name.as_str())
                        .unwrap_or("?");
                    (
                        format!("{} enters ({:?})", name, side),
                        egui::Color32::from_rgb(180, 180, 180),
                    )
                }
                _ => continue,
            };
            self.log.push((text, color));
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
    frame_count.0 += 1;
    if frame_count.0 < 3 {
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else { return };

    if state.result.is_none() {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.centered_and_justified(|ui| {
                ui.heading("No battle loaded");
            });
        });
        return;
    }

    let total = state.result.as_ref().unwrap().actions.len();
    let winner = state.result.as_ref().unwrap().winner;
    let turns = state.result.as_ref().unwrap().turns;

    // Auto-play
    if state.playing && state.action_index < total {
        state.timer += time.delta_secs();
        if state.timer >= state.speed {
            state.timer = 0.0;
            state.action_index += 1;
            state.rebuild_state();
        }
    }

    // === TOP BAR ===
    egui::TopBottomPanel::top("battle_top").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.heading("Battle");
            ui.separator();
            let (label, color) = match winner {
                BattleSide::Left => ("Left Wins!", egui::Color32::from_rgb(100, 255, 100)),
                BattleSide::Right => ("Right Wins!", egui::Color32::from_rgb(255, 100, 100)),
            };
            ui.colored_label(color, label);
            ui.label(format!("• {} turns • {} actions", turns, total));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Exit").clicked() {
                    state.result = None;
                    next_state.set(GameState::Home);
                }
            });
        });
    });

    // === BOTTOM CONTROLS ===
    egui::TopBottomPanel::bottom("battle_controls").show(ctx, |ui| {
        ui.horizontal(|ui| {
            if ui.button("⏮ Start").clicked() {
                state.action_index = 0;
                state.rebuild_state();
            }
            if ui.button("◀ Step").clicked() {
                state.action_index = state.action_index.saturating_sub(1);
                state.rebuild_state();
            }
            if state.playing {
                if ui.button("⏸ Pause").clicked() {
                    state.playing = false;
                }
            } else if ui.button("▶ Play").clicked() {
                state.playing = true;
            }
            if ui.button("Step ▶").clicked() {
                state.action_index = (state.action_index + 1).min(total);
                state.rebuild_state();
            }
            if ui.button("End ⏭").clicked() {
                state.action_index = total;
                state.rebuild_state();
            }
            ui.separator();
            ui.label(format!("{}/{}", state.action_index, total));
            ui.separator();
            ui.label("Speed:");
            ui.add(egui::Slider::new(&mut state.speed, 0.02..=1.0).logarithmic(true));
        });
    });

    // === MAIN AREA ===
    egui::CentralPanel::default().show(ctx, |ui| {
        let avail = ui.available_rect_before_wrap();

        // Split: left half = units, right half = log
        let mid_x = avail.left() + avail.width() * 0.55;

        // --- UNIT DISPLAY ---
        let unit_area = egui::Rect::from_min_max(avail.min, egui::pos2(mid_x, avail.max.y));

        // Background
        ui.painter()
            .rect_filled(unit_area, 4.0, egui::Color32::from_rgb(15, 15, 25));

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

        let unit_size = (unit_area.height() / 5.5).min(90.0);
        let left_x = unit_area.left() + unit_area.width() * 0.25;
        let right_x = unit_area.left() + unit_area.width() * 0.75;

        for (i, unit) in left_units.iter().enumerate() {
            let y = unit_area.top()
                + 20.0
                + (i as f32 + 0.5) * (unit_area.height() - 40.0) / left_units.len().max(1) as f32;
            let rect = egui::Rect::from_center_size(
                egui::pos2(left_x, y),
                egui::vec2(unit_size, unit_size),
            );
            let alpha = if unit.alive { 255 } else { 50 };
            let c = egui::Color32::from_rgba_premultiplied(
                unit.color.r(),
                unit.color.g(),
                unit.color.b(),
                alpha,
            );
            paint_default_unit(rect, c, unit.hp, unit.pwr, unit.dmg, &unit.name, ui);
        }

        for (i, unit) in right_units.iter().enumerate() {
            let y = unit_area.top()
                + 20.0
                + (i as f32 + 0.5) * (unit_area.height() - 40.0) / right_units.len().max(1) as f32;
            let rect = egui::Rect::from_center_size(
                egui::pos2(right_x, y),
                egui::vec2(unit_size, unit_size),
            );
            let alpha = if unit.alive { 255 } else { 50 };
            let c = egui::Color32::from_rgba_premultiplied(
                unit.color.r(),
                unit.color.g(),
                unit.color.b(),
                alpha,
            );
            paint_default_unit(rect, c, unit.hp, unit.pwr, unit.dmg, &unit.name, ui);
        }

        // VS label
        ui.painter().text(
            unit_area.center(),
            egui::Align2::CENTER_CENTER,
            "VS",
            egui::FontId::proportional(28.0),
            egui::Color32::from_rgb(100, 100, 100),
        );

        // --- BATTLE LOG ---
        let log_area = egui::Rect::from_min_max(egui::pos2(mid_x + 8.0, avail.top()), avail.max);
        ui.allocate_new_ui(egui::UiBuilder::new().max_rect(log_area), |ui| {
            ui.heading("Battle Log");
            ui.separator();
            egui::ScrollArea::vertical()
                .stick_to_bottom(true)
                .max_height(log_area.height() - 40.0)
                .show(ui, |ui| {
                    for (text, color) in &state.log {
                        ui.colored_label(*color, text);
                    }
                    if state.log.is_empty() {
                        ui.colored_label(egui::Color32::GRAY, "Press ▶ Play to start...");
                    }
                });
        });
    });
}
