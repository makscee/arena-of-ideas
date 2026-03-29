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

// ===== Visual Effect on the Timeline =====

#[derive(Clone, Debug)]
struct VisualEffect {
    start: f32,
    duration: f32,
    kind: EffectKind,
}

#[derive(Clone, Debug)]
enum EffectKind {
    /// Line from source unit to target unit
    Line {
        source_id: u64,
        target_id: u64,
        color: egui::Color32,
        label: String,
    },
    /// Popup text on a unit
    Popup {
        unit_id: u64,
        text: String,
        color: egui::Color32,
    },
    /// Flash a unit's color
    Flash { unit_id: u64 },
    /// Action card text between teams, positioned at source unit's column
    Card {
        text: String,
        color: egui::Color32,
        source_id: u64,
    },
}

// ===== Unit Visual =====

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

impl BattleUnitVisual {
    pub fn new(
        id: u64,
        name: String,
        hp: i32,
        pwr: i32,
        side: BattleSide,
        slot: u8,
        color: egui::Color32,
    ) -> Self {
        Self {
            id,
            name,
            initial_hp: hp,
            initial_pwr: pwr,
            hp,
            pwr,
            dmg: 0,
            alive: true,
            side,
            slot,
            color,
        }
    }
}

// ===== Scene State =====

#[derive(Resource, Default)]
pub struct BattleSceneState {
    pub result: Option<BattleResult>,
    pub initial_units: Vec<BattleUnitVisual>,
    pub units: Vec<BattleUnitVisual>,
    pub effects: Vec<VisualEffect>,
    pub total_duration: f32,
    pub time: f32,
    pub playing: bool,
    pub speed: f32,
}

impl BattleSceneState {
    pub fn load(&mut self, result: BattleResult, units: Vec<BattleUnitVisual>) {
        let effects = build_effects(&result.actions, &units);
        let total_duration = effects.last().map(|e| e.start + e.duration).unwrap_or(0.0) + 0.5;
        self.initial_units = units.clone();
        self.units = units;
        self.effects = effects;
        self.total_duration = total_duration;
        self.result = Some(result);
        self.time = 0.0;
        self.playing = false;
        self.speed = 1.0;
    }

    fn rebuild_units_at_time(&mut self) {
        self.units = self.initial_units.clone();
        let Some(ref result) = self.result else {
            return;
        };

        for action in &result.actions {
            let action_time = self
                .effects
                .iter()
                .find(|e| {
                    matches!(&e.kind, EffectKind::Card { .. })
                        || matches!(&e.kind, EffectKind::Line { .. })
                })
                .map(|_| 0.0) // we use a simpler approach below
                .unwrap_or(0.0);
            let _ = action_time; // unused, we just apply all actions up to time
        }

        // Re-derive: apply actions whose effects have started
        self.units = self.initial_units.clone();
        let actions = self.result.as_ref().unwrap().actions.clone();
        let mut action_idx = 0;
        for effect in &self.effects {
            if effect.start > self.time {
                break;
            }
            // Only Card effects correspond to actual state changes
            if matches!(&effect.kind, EffectKind::Card { text, .. } if !text.is_empty())
                || matches!(&effect.kind, EffectKind::Popup { .. })
            {
                // Find the next unprocessed action that generates a card
                while action_idx < actions.len() {
                    let applied = apply_action_to_units(&actions[action_idx], &mut self.units);
                    action_idx += 1;
                    if applied {
                        break;
                    }
                }
            }
        }
    }
}

fn apply_action_to_units(action: &BattleAction, units: &mut [BattleUnitVisual]) -> bool {
    match action {
        BattleAction::Damage { target, amount, .. } => {
            if let Some(u) = units.iter_mut().find(|u| u.id == *target) {
                u.dmg += amount;
                if u.hp - u.dmg <= 0 {
                    u.alive = false;
                }
            }
            true
        }
        BattleAction::Heal { target, amount, .. } => {
            if let Some(u) = units.iter_mut().find(|u| u.id == *target) {
                u.dmg = (u.dmg - amount).max(0);
            }
            true
        }
        BattleAction::Death { unit } => {
            if let Some(u) = units.iter_mut().find(|u| u.id == *unit) {
                u.alive = false;
            }
            true
        }
        BattleAction::StatChange { unit, stat, delta } => {
            if let Some(u) = units.iter_mut().find(|u| u.id == *unit) {
                match stat {
                    shared::battle::StatKind::Pwr => u.pwr = (u.pwr + delta).max(0),
                    shared::battle::StatKind::Hp => u.hp = (u.hp + delta).max(0),
                    _ => {}
                }
            }
            true
        }
        BattleAction::Fatigue { amount } => {
            for u in units.iter_mut().filter(|u| u.alive) {
                u.dmg += amount;
            }
            true
        }
        _ => false, // Spawn, AbilityUsed, etc. don't change state
    }
}

fn unit_name(units: &[BattleUnitVisual], id: u64) -> String {
    units
        .iter()
        .find(|u| u.id == id)
        .map(|u| u.name.clone())
        .unwrap_or_else(|| format!("#{}", id))
}

/// Build visual effects timeline from battle actions.
fn build_effects(actions: &[BattleAction], units: &[BattleUnitVisual]) -> Vec<VisualEffect> {
    let mut effects = Vec::new();
    let mut t = 0.0;

    for action in actions {
        match action {
            BattleAction::Spawn { .. } => {}
            BattleAction::AbilityUsed {
                source,
                ability_name,
            } => {
                let name = unit_name(units, *source);
                effects.push(VisualEffect {
                    start: t,
                    duration: 1.5,
                    kind: EffectKind::Card {
                        text: format!("⚔ {} uses {}", name, ability_name),
                        color: egui::Color32::from_rgb(100, 170, 255),
                        source_id: *source,
                    },
                });
                t += 0.3;
            }
            BattleAction::Damage {
                source,
                target,
                amount,
            } => {
                // Arrow: 0.3s, card: 1.5s
                effects.push(VisualEffect {
                    start: t,
                    duration: 0.3,
                    kind: EffectKind::Line {
                        source_id: *source,
                        target_id: *target,
                        color: egui::Color32::from_rgb(255, 80, 80),
                        label: format!("{}", amount),
                    },
                });
                effects.push(VisualEffect {
                    start: t + 0.1,
                    duration: 0.3,
                    kind: EffectKind::Flash { unit_id: *target },
                });
                effects.push(VisualEffect {
                    start: t + 0.1,
                    duration: 0.8,
                    kind: EffectKind::Popup {
                        unit_id: *target,
                        text: format!("-{}", amount),
                        color: egui::Color32::from_rgb(255, 80, 80),
                    },
                });
                effects.push(VisualEffect {
                    start: t,
                    duration: 1.5,
                    kind: EffectKind::Card {
                        text: format!(
                            "{} → {} → {}",
                            unit_name(units, *source),
                            amount,
                            unit_name(units, *target)
                        ),
                        color: egui::Color32::from_rgb(255, 120, 120),
                        source_id: *source,
                    },
                });
                t += 0.5;
            }
            BattleAction::Heal {
                source,
                target,
                amount,
            } => {
                effects.push(VisualEffect {
                    start: t,
                    duration: 0.3,
                    kind: EffectKind::Line {
                        source_id: *source,
                        target_id: *target,
                        color: egui::Color32::from_rgb(80, 230, 80),
                        label: format!("+{}", amount),
                    },
                });
                effects.push(VisualEffect {
                    start: t + 0.1,
                    duration: 0.8,
                    kind: EffectKind::Popup {
                        unit_id: *target,
                        text: format!("+{}", amount),
                        color: egui::Color32::from_rgb(80, 255, 80),
                    },
                });
                effects.push(VisualEffect {
                    start: t,
                    duration: 1.5,
                    kind: EffectKind::Card {
                        text: format!(
                            "✚ {} → +{} → {}",
                            unit_name(units, *source),
                            amount,
                            unit_name(units, *target)
                        ),
                        color: egui::Color32::from_rgb(80, 220, 80),
                        source_id: *source,
                    },
                });
                t += 0.6;
            }
            BattleAction::Death { unit } => {
                let name = unit_name(units, *unit);
                effects.push(VisualEffect {
                    start: t,
                    duration: 1.5,
                    kind: EffectKind::Card {
                        text: format!("☠ {} has fallen!", name),
                        color: egui::Color32::from_rgb(255, 50, 50),
                        source_id: *unit,
                    },
                });
                t += 0.8;
            }
            BattleAction::Fatigue { amount } => {
                effects.push(VisualEffect {
                    start: t,
                    duration: 1.5,
                    kind: EffectKind::Card {
                        text: format!("⚡ FATIGUE! All take {} dmg", amount),
                        color: egui::Color32::from_rgb(255, 200, 50),
                        source_id: 0,
                    },
                });
                t += 0.7;
            }
            BattleAction::StatChange { unit, stat, delta } => {
                effects.push(VisualEffect {
                    start: t,
                    duration: 0.6,
                    kind: EffectKind::Popup {
                        unit_id: *unit,
                        text: format!("{:?}{:+}", stat, delta),
                        color: egui::Color32::from_rgb(200, 200, 100),
                    },
                });
                t += 0.2;
            }
            _ => {}
        }
    }

    effects
}

// ===== Rendering =====

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

    let total_dur = state.total_duration;
    let winner = state.result.as_ref().unwrap().winner;
    let turns = state.result.as_ref().unwrap().turns;

    // Advance time
    let prev_time = state.time;
    if state.playing && state.time < total_dur {
        state.time += time.delta_secs() * state.speed;
        if state.time > total_dur {
            state.time = total_dur;
        }
    }

    // Rebuild unit state when time changes
    if (state.time - prev_time).abs() > 0.001 || frame_count.0 == 3 {
        state.rebuild_units_at_time();
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
            ui.label(format!("• {} turns", turns));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Exit").clicked() {
                    state.result = None;
                    next_state.set(GameState::Home);
                }
            });
        });
    });

    // === MAIN AREA (controls rendered inside at bottom) ===
    egui::CentralPanel::default().show(ctx, |ui| {
        let avail = ui.available_rect_before_wrap();
        let painter = ui.painter().clone();

        // Background
        painter.rect_filled(avail, 4.0, egui::Color32::from_rgb(12, 12, 20));

        // Layout: top row = left team, middle = action cards, bottom row = right team
        let row_height = (avail.height() * 0.25).min(100.0);
        let unit_size = (row_height * 0.85).min(80.0);

        let top_y = avail.top() + row_height * 0.5;
        let bottom_y = avail.bottom() - row_height * 0.5;
        let mid_y = (top_y + bottom_y) * 0.5;

        let left_units: Vec<&BattleUnitVisual> = state
            .units
            .iter()
            .filter(|u| u.side == BattleSide::Left)
            .collect();
        let right_units: Vec<&BattleUnitVisual> = state
            .units
            .iter()
            .filter(|u| u.side == BattleSide::Right)
            .collect();

        // Unit positions (stored for line drawing)
        let mut unit_positions: std::collections::HashMap<u64, egui::Pos2> =
            std::collections::HashMap::new();

        // Draw left team (top row)
        for (i, unit) in left_units.iter().enumerate() {
            let x =
                avail.left() + (i as f32 + 0.5) * avail.width() / left_units.len().max(1) as f32;
            let pos = egui::pos2(x, top_y);
            unit_positions.insert(unit.id, pos);

            let alpha = if unit.alive { 255 } else { 60 };
            let c = egui::Color32::from_rgba_premultiplied(
                unit.color.r(),
                unit.color.g(),
                unit.color.b(),
                alpha,
            );
            let sz = if unit.alive {
                unit_size
            } else {
                unit_size * 0.6
            };
            let rect = egui::Rect::from_center_size(pos, egui::vec2(sz, sz));
            paint_default_unit(rect, c, unit.hp, unit.pwr, unit.dmg, &unit.name, ui);
        }

        // Draw right team (bottom row)
        for (i, unit) in right_units.iter().enumerate() {
            let x =
                avail.left() + (i as f32 + 0.5) * avail.width() / right_units.len().max(1) as f32;
            let pos = egui::pos2(x, bottom_y);
            unit_positions.insert(unit.id, pos);

            let alpha = if unit.alive { 255 } else { 60 };
            let c = egui::Color32::from_rgba_premultiplied(
                unit.color.r(),
                unit.color.g(),
                unit.color.b(),
                alpha,
            );
            let sz = if unit.alive {
                unit_size
            } else {
                unit_size * 0.6
            };
            let rect = egui::Rect::from_center_size(pos, egui::vec2(sz, sz));
            paint_default_unit(rect, c, unit.hp, unit.pwr, unit.dmg, &unit.name, ui);
        }

        // Team labels
        painter.text(
            egui::pos2(avail.left() + 10.0, avail.top() + 5.0),
            egui::Align2::LEFT_TOP,
            "Left Team",
            egui::FontId::proportional(14.0),
            egui::Color32::from_rgb(150, 150, 150),
        );
        painter.text(
            egui::pos2(avail.left() + 10.0, avail.bottom() - row_height - 5.0),
            egui::Align2::LEFT_TOP,
            "Right Team",
            egui::FontId::proportional(14.0),
            egui::Color32::from_rgb(150, 150, 150),
        );

        // Draw active effects
        // Track how many cards are active per source_id for stacking
        let mut card_count_per_source: std::collections::HashMap<u64, usize> =
            std::collections::HashMap::new();
        let current_time = state.time;
        for effect in &state.effects {
            let progress = (current_time - effect.start) / effect.duration;
            if progress < 0.0 || progress > 1.0 {
                continue;
            }

            let fade = if progress > 0.7 {
                (1.0 - progress) / 0.3
            } else {
                1.0
            };
            let alpha = (fade * 255.0) as u8;

            match &effect.kind {
                EffectKind::Line {
                    source_id,
                    target_id,
                    color,
                    label,
                } => {
                    if let (Some(&from), Some(&to)) =
                        (unit_positions.get(source_id), unit_positions.get(target_id))
                    {
                        let c = egui::Color32::from_rgba_premultiplied(
                            color.r(),
                            color.g(),
                            color.b(),
                            alpha,
                        );

                        // Animated line: grows from source to target
                        let line_progress = (progress * 2.0).min(1.0);
                        let current_to = egui::pos2(
                            from.x + (to.x - from.x) * line_progress,
                            from.y + (to.y - from.y) * line_progress,
                        );

                        // Main line (thick, glowing)
                        painter.line_segment([from, current_to], egui::Stroke::new(3.0, c));
                        // Glow line
                        let glow = egui::Color32::from_rgba_premultiplied(
                            color.r(),
                            color.g(),
                            color.b(),
                            alpha / 3,
                        );
                        painter.line_segment([from, current_to], egui::Stroke::new(8.0, glow));

                        // Arrowhead at current_to
                        if line_progress > 0.1 {
                            let dir = egui::vec2(to.x - from.x, to.y - from.y);
                            let len = (dir.x * dir.x + dir.y * dir.y).sqrt();
                            if len > 0.0 {
                                let norm = egui::vec2(dir.x / len, dir.y / len);
                                let perp = egui::vec2(-norm.y, norm.x);
                                let arrow_size = 10.0;
                                let tip = current_to;
                                let left = egui::pos2(
                                    tip.x - norm.x * arrow_size + perp.x * arrow_size * 0.5,
                                    tip.y - norm.y * arrow_size + perp.y * arrow_size * 0.5,
                                );
                                let right = egui::pos2(
                                    tip.x - norm.x * arrow_size - perp.x * arrow_size * 0.5,
                                    tip.y - norm.y * arrow_size - perp.y * arrow_size * 0.5,
                                );
                                painter.line_segment([left, tip], egui::Stroke::new(3.0, c));
                                painter.line_segment([right, tip], egui::Stroke::new(3.0, c));
                            }
                        }

                        // Label at midpoint with background
                        if line_progress > 0.3 {
                            let mid = egui::pos2(
                                (from.x + current_to.x) * 0.5,
                                (from.y + current_to.y) * 0.5,
                            );
                            let label_bg = egui::Color32::from_rgba_premultiplied(
                                0,
                                0,
                                0,
                                (alpha as u16 * 3 / 4) as u8,
                            );
                            let label_rect =
                                egui::Rect::from_center_size(mid, egui::vec2(40.0, 22.0));
                            painter.rect_filled(label_rect, 4.0, label_bg);
                            painter.text(
                                mid,
                                egui::Align2::CENTER_CENTER,
                                label,
                                egui::FontId::proportional(16.0),
                                c,
                            );
                        }
                    }
                }
                EffectKind::Popup {
                    unit_id,
                    text,
                    color,
                } => {
                    if let Some(&pos) = unit_positions.get(unit_id) {
                        let c = egui::Color32::from_rgba_premultiplied(
                            color.r(),
                            color.g(),
                            color.b(),
                            alpha,
                        );
                        // Float upward
                        let offset_y = -unit_size * 0.5 - progress * 30.0;
                        let scale = 1.0 + progress * 0.3; // grow slightly
                        painter.text(
                            egui::pos2(pos.x, pos.y + offset_y),
                            egui::Align2::CENTER_CENTER,
                            text,
                            egui::FontId::proportional(18.0 * scale),
                            c,
                        );
                    }
                }
                EffectKind::Flash { unit_id } => {
                    if let Some(&pos) = unit_positions.get(unit_id) {
                        let flash_alpha = (alpha as f32 * 0.5 * (1.0 - progress)) as u8;
                        painter.circle_filled(
                            pos,
                            unit_size * 0.45 * (1.0 + progress * 0.2),
                            egui::Color32::from_rgba_premultiplied(255, 255, 255, flash_alpha),
                        );
                    }
                }
                EffectKind::Card {
                    text,
                    color,
                    source_id,
                } => {
                    if text.is_empty() {
                        continue;
                    }
                    let c = egui::Color32::from_rgba_premultiplied(
                        color.r(),
                        color.g(),
                        color.b(),
                        alpha,
                    );

                    // Position halfway between unit and center, stack if multiple active
                    let stack_idx = card_count_per_source.entry(*source_id).or_insert(0);
                    let stack_offset = *stack_idx as f32 * 36.0;
                    *card_count_per_source.get_mut(source_id).unwrap() += 1;

                    let (card_x, card_y) = if *source_id == 0 {
                        (avail.center().x, mid_y + stack_offset)
                    } else if let Some(&pos) = unit_positions.get(source_id) {
                        // Halfway between unit and center
                        let half_y = (pos.y + mid_y) * 0.5;
                        (pos.x, half_y + stack_offset)
                    } else {
                        (avail.center().x, mid_y + stack_offset)
                    };

                    // Card background
                    let card_w = (text.len() as f32 * 9.0).max(140.0).min(300.0);
                    let card_h = 32.0;
                    let card_rect = egui::Rect::from_center_size(
                        egui::pos2(card_x, card_y),
                        egui::vec2(card_w, card_h),
                    );
                    let bg_alpha = (alpha as u16 * 3 / 4) as u8;
                    painter.rect_filled(
                        card_rect,
                        8.0,
                        egui::Color32::from_rgba_premultiplied(12, 12, 22, bg_alpha),
                    );
                    // Colored border
                    painter.rect_stroke(
                        card_rect,
                        8.0,
                        egui::Stroke::new(
                            2.0,
                            egui::Color32::from_rgba_premultiplied(
                                color.r(),
                                color.g(),
                                color.b(),
                                alpha / 2,
                            ),
                        ),
                        egui::StrokeKind::Outside,
                    );
                    // Left accent bar
                    let accent = egui::Rect::from_min_size(
                        egui::pos2(card_rect.left() + 3.0, card_rect.top() + 5.0),
                        egui::vec2(3.0, card_h - 10.0),
                    );
                    painter.rect_filled(accent, 1.5, c);
                    // Text
                    painter.text(
                        egui::pos2(card_x + 4.0, card_y),
                        egui::Align2::CENTER_CENTER,
                        text,
                        egui::FontId::proportional(13.0),
                        c,
                    );
                }
            }
        }

        // === PLAYBACK CONTROLS at bottom of battle area ===
        let controls_height = 50.0;
        let controls_rect = egui::Rect::from_min_size(
            egui::pos2(avail.left(), avail.bottom() - controls_height),
            egui::vec2(avail.width(), controls_height),
        );

        // Semi-transparent background
        painter.rect_filled(
            controls_rect,
            0.0,
            egui::Color32::from_rgba_premultiplied(10, 10, 18, 200),
        );

        ui.allocate_new_ui(egui::UiBuilder::new().max_rect(controls_rect), |ui| {
            // Full-width progress slider
            let mut t = state.time;
            let slider = egui::Slider::new(&mut t, 0.0..=total_dur.max(0.01)).show_value(false);
            if ui.add_sized([ui.available_width(), 18.0], slider).changed() {
                state.time = t;
                state.rebuild_units_at_time();
            }
            // Buttons row
            ui.horizontal(|ui| {
                if ui.button("⏮").clicked() {
                    state.time = 0.0;
                    state.rebuild_units_at_time();
                }
                if ui.button("⏪").clicked() {
                    state.time = (state.time - 1.0).max(0.0);
                    state.rebuild_units_at_time();
                }
                if state.playing {
                    if ui.button("⏸").clicked() {
                        state.playing = false;
                    }
                } else if ui.button("▶").clicked() {
                    if state.time >= total_dur {
                        state.time = 0.0;
                        state.rebuild_units_at_time();
                    }
                    state.playing = true;
                }
                if ui.button("⏩").clicked() {
                    state.time = (state.time + 1.0).min(total_dur);
                    state.rebuild_units_at_time();
                }
                if ui.button("⏭").clicked() {
                    state.time = total_dur;
                    state.rebuild_units_at_time();
                }
                ui.separator();
                ui.label(format!("{:.1}s / {:.1}s", state.time, total_dur));
                ui.separator();
                ui.label("Speed:");
                let speeds = [1.0, 2.0, 4.0];
                for &s in &speeds {
                    let active = (state.speed - s).abs() < 0.01;
                    if ui
                        .selectable_label(active, format!("x{}", s as i32))
                        .clicked()
                    {
                        state.speed = s;
                    }
                }
            });
        });
    });
}
