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

// ===== Animated Action Timeline =====

/// An action placed on a timeline with a start time and duration.
#[derive(Clone, Debug)]
struct TimedAction {
    action: BattleAction,
    start: f32,
    duration: f32,
}

/// Build a timeline from battle actions: each action gets a start time.
fn build_timeline(actions: &[BattleAction]) -> (Vec<TimedAction>, f32) {
    let mut timeline = Vec::new();
    let mut t = 0.0;

    for action in actions {
        let duration = match action {
            BattleAction::Spawn { .. } => 0.0, // instant
            BattleAction::AbilityUsed { .. } => 0.1,
            BattleAction::Damage { .. } => 0.4,
            BattleAction::Heal { .. } => 0.3,
            BattleAction::Death { .. } => 0.5,
            BattleAction::StatChange { .. } => 0.15,
            BattleAction::Fatigue { .. } => 0.3,
            BattleAction::Vfx { .. } => 0.2,
            BattleAction::Wait { seconds } => *seconds,
        };

        timeline.push(TimedAction {
            action: action.clone(),
            start: t,
            duration,
        });

        t += duration;
    }

    (timeline, t)
}

// ===== Unit Visual State =====

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
    // Animation state
    pub offset_x: f32,                    // horizontal offset for strike animation
    pub flash_timer: f32,                 // damage flash countdown
    pub death_timer: f32,                 // death fade (1.0 = alive, 0.0 = gone)
    pub damage_popup: Option<(i32, f32)>, // (amount, timer)
    pub heal_popup: Option<(i32, f32)>,
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
            offset_x: 0.0,
            flash_timer: 0.0,
            death_timer: 1.0,
            damage_popup: None,
            heal_popup: None,
        }
    }
}

// ===== Scene State =====

#[derive(Resource, Default)]
pub struct BattleSceneState {
    pub result: Option<BattleResult>,
    pub initial_units: Vec<BattleUnitVisual>,
    pub units: Vec<BattleUnitVisual>,
    pub timeline: Vec<TimedAction>,
    pub total_duration: f32,
    pub time: f32,
    pub playing: bool,
    pub speed: f32,
    pub log: Vec<(String, egui::Color32)>,
    pub last_processed: usize, // last action index applied
}

impl BattleSceneState {
    pub fn load(&mut self, result: BattleResult, units: Vec<BattleUnitVisual>) {
        let (timeline, total_duration) = build_timeline(&result.actions);
        self.initial_units = units.clone();
        self.units = units;
        self.timeline = timeline;
        self.total_duration = total_duration;
        self.result = Some(result);
        self.time = 0.0;
        self.playing = false;
        self.speed = 1.0;
        self.log.clear();
        self.last_processed = 0;
    }

    /// Rebuild from scratch up to current time.
    fn rebuild_to_time(&mut self) {
        self.units = self.initial_units.clone();
        self.log.clear();
        self.last_processed = 0;

        let actions: Vec<(f32, BattleAction)> = self
            .timeline
            .iter()
            .map(|t| (t.start, t.action.clone()))
            .collect();

        for (start, action) in &actions {
            if *start > self.time {
                break;
            }
            self.apply_action(action);
            self.last_processed += 1;
        }

        // Animate the current action if we're in the middle of one
        self.update_animations();
    }

    fn apply_action(&mut self, action: &BattleAction) {
        match action {
            BattleAction::Damage {
                source,
                target,
                amount,
            } => {
                if let Some(u) = self.units.iter_mut().find(|u| u.id == *target) {
                    u.dmg += amount;
                    u.flash_timer = 0.3;
                    u.damage_popup = Some((*amount, 0.5));
                }
                // Strike animation: source slides toward target
                if let Some(u) = self.units.iter_mut().find(|u| u.id == *source) {
                    let dir = if u.side == BattleSide::Left {
                        1.0
                    } else {
                        -1.0
                    };
                    u.offset_x = dir * 30.0;
                }
                let sname = self.unit_name(*source);
                let tname = self.unit_name(*target);
                self.log.push((
                    format!("{} → {} dmg → {}", sname, amount, tname),
                    egui::Color32::from_rgb(255, 120, 120),
                ));
            }
            BattleAction::Heal {
                source,
                target,
                amount,
            } => {
                if let Some(u) = self.units.iter_mut().find(|u| u.id == *target) {
                    u.dmg = (u.dmg - amount).max(0);
                    u.heal_popup = Some((*amount, 0.5));
                }
                let sname = self.unit_name(*source);
                let tname = self.unit_name(*target);
                self.log.push((
                    format!("{} heals {} +{}", sname, tname, amount),
                    egui::Color32::from_rgb(120, 255, 120),
                ));
            }
            BattleAction::Death { unit } => {
                if let Some(u) = self.units.iter_mut().find(|u| u.id == *unit) {
                    u.alive = false;
                    u.death_timer = 0.0;
                }
                let name = self.unit_name(*unit);
                self.log.push((
                    format!("{} dies!", name),
                    egui::Color32::from_rgb(255, 50, 50),
                ));
            }
            BattleAction::StatChange { unit, stat, delta } => {
                if let Some(u) = self.units.iter_mut().find(|u| u.id == *unit) {
                    match stat {
                        shared::battle::StatKind::Pwr => u.pwr = (u.pwr + delta).max(0),
                        shared::battle::StatKind::Hp => u.hp = (u.hp + delta).max(0),
                        _ => {}
                    }
                }
            }
            BattleAction::AbilityUsed {
                source,
                ability_name,
            } => {
                let name = self.unit_name(*source);
                self.log.push((
                    format!("{} uses {}", name, ability_name),
                    egui::Color32::from_rgb(120, 180, 255),
                ));
            }
            BattleAction::Fatigue { amount } => {
                for u in self.units.iter_mut().filter(|u| u.alive) {
                    u.dmg += amount;
                    u.flash_timer = 0.2;
                }
                self.log.push((
                    format!("FATIGUE! All take {} dmg", amount),
                    egui::Color32::from_rgb(255, 200, 50),
                ));
            }
            BattleAction::Spawn { unit, side, .. } => {
                let name = self.unit_name(*unit);
                self.log.push((
                    format!("{} enters ({:?})", name, side),
                    egui::Color32::from_rgb(150, 150, 150),
                ));
            }
            _ => {}
        }
    }

    fn update_animations(&mut self) {
        let dt = 0.016; // ~60fps frame
        for u in &mut self.units {
            // Slide back from strike
            u.offset_x *= 0.8;
            if u.offset_x.abs() < 0.5 {
                u.offset_x = 0.0;
            }

            // Flash decay
            u.flash_timer = (u.flash_timer - dt).max(0.0);

            // Death fade
            if !u.alive && u.death_timer < 1.0 {
                u.death_timer = (u.death_timer + dt * 3.0).min(1.0);
            }

            // Popup decay
            if let Some((_, ref mut t)) = u.damage_popup {
                *t -= dt;
                if *t <= 0.0 {
                    u.damage_popup = None;
                }
            }
            if let Some((_, ref mut t)) = u.heal_popup {
                *t -= dt;
                if *t <= 0.0 {
                    u.heal_popup = None;
                }
            }
        }
    }

    fn unit_name(&self, id: u64) -> String {
        self.units
            .iter()
            .find(|u| u.id == id)
            .map(|u| u.name.clone())
            .unwrap_or_else(|| format!("#{}", id))
    }
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
    let total_actions = state.result.as_ref().unwrap().actions.len();

    // Advance time
    if state.playing && state.time < total_dur {
        state.time += time.delta_secs() * state.speed;
        if state.time > total_dur {
            state.time = total_dur;
        }

        // Apply new actions that are now in range
        while state.last_processed < state.timeline.len() {
            if state.timeline[state.last_processed].start <= state.time {
                let action = state.timeline[state.last_processed].action.clone();
                state.apply_action(&action);
                state.last_processed += 1;
            } else {
                break;
            }
        }
    }

    // Always update animations
    state.update_animations();

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
            ui.label(format!("• {} turns • {} actions", turns, total_actions));
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
            if ui.button("⏮").clicked() {
                state.time = 0.0;
                state.rebuild_to_time();
            }
            if state.playing {
                if ui.button("⏸ Pause").clicked() {
                    state.playing = false;
                }
            } else {
                if ui.button("▶ Play").clicked() {
                    if state.time >= total_dur {
                        state.time = 0.0;
                        state.rebuild_to_time();
                    }
                    state.playing = true;
                }
            }
            if ui.button("⏭").clicked() {
                state.time = total_dur;
                state.rebuild_to_time();
            }
            ui.separator();

            // Time slider
            let mut t = state.time;
            let response = ui.add(
                egui::Slider::new(&mut t, 0.0..=total_dur.max(0.01))
                    .text("time")
                    .show_value(true),
            );
            if response.changed() {
                state.time = t;
                state.rebuild_to_time();
            }

            ui.separator();
            ui.label("Speed:");
            ui.add(egui::Slider::new(&mut state.speed, 0.25..=4.0).logarithmic(true));
        });
    });

    // === MAIN AREA ===
    egui::CentralPanel::default().show(ctx, |ui| {
        let avail = ui.available_rect_before_wrap();
        let mid_x = avail.left() + avail.width() * 0.55;

        // --- BATTLEFIELD ---
        let bf = egui::Rect::from_min_max(avail.min, egui::pos2(mid_x, avail.max.y));
        ui.painter()
            .rect_filled(bf, 4.0, egui::Color32::from_rgb(12, 12, 20));

        // Dividing line
        ui.painter().line_segment(
            [
                egui::pos2(bf.center().x, bf.top() + 10.0),
                egui::pos2(bf.center().x, bf.bottom() - 10.0),
            ],
            egui::Stroke::new(1.0, egui::Color32::from_rgb(40, 40, 60)),
        );

        let unit_size = (bf.height() / 6.0).min(80.0);
        let left_x = bf.left() + bf.width() * 0.25;
        let right_x = bf.left() + bf.width() * 0.75;

        // Render units
        let units_snapshot: Vec<BattleUnitVisual> = state.units.clone();
        let left_units: Vec<&BattleUnitVisual> = units_snapshot
            .iter()
            .filter(|u| u.side == BattleSide::Left)
            .collect();
        let right_units: Vec<&BattleUnitVisual> = units_snapshot
            .iter()
            .filter(|u| u.side == BattleSide::Right)
            .collect();

        for (i, unit) in left_units.iter().enumerate() {
            let base_y = bf.top()
                + 20.0
                + (i as f32 + 0.5) * (bf.height() - 40.0) / left_units.len().max(1) as f32;
            let x = left_x + unit.offset_x;
            render_unit(ui, unit, egui::pos2(x, base_y), unit_size);
        }

        for (i, unit) in right_units.iter().enumerate() {
            let base_y = bf.top()
                + 20.0
                + (i as f32 + 0.5) * (bf.height() - 40.0) / right_units.len().max(1) as f32;
            let x = right_x + unit.offset_x;
            render_unit(ui, unit, egui::pos2(x, base_y), unit_size);
        }

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

fn render_unit(ui: &mut egui::Ui, unit: &BattleUnitVisual, center: egui::Pos2, size: f32) {
    let alpha = if unit.alive {
        255
    } else {
        (255.0 * (1.0 - unit.death_timer) * 0.3) as u8
    };

    // Flash effect on damage
    let color = if unit.flash_timer > 0.0 {
        egui::Color32::from_rgba_premultiplied(255, 255, 255, alpha)
    } else {
        egui::Color32::from_rgba_premultiplied(
            unit.color.r(),
            unit.color.g(),
            unit.color.b(),
            alpha,
        )
    };

    let actual_size = if unit.alive {
        size
    } else {
        size * (1.0 - unit.death_timer * 0.3)
    };
    let rect = egui::Rect::from_center_size(center, egui::vec2(actual_size, actual_size));

    paint_default_unit(rect, color, unit.hp, unit.pwr, unit.dmg, &unit.name, ui);

    // Damage popup
    if let Some((amount, timer)) = unit.damage_popup {
        let popup_y = center.y - size * 0.5 - (0.5 - timer) * 40.0;
        let popup_alpha = (timer * 2.0 * 255.0).min(255.0) as u8;
        ui.painter().text(
            egui::pos2(center.x, popup_y),
            egui::Align2::CENTER_CENTER,
            format!("-{}", amount),
            egui::FontId::proportional(size * 0.25),
            egui::Color32::from_rgba_premultiplied(255, 80, 80, popup_alpha),
        );
    }

    // Heal popup
    if let Some((amount, timer)) = unit.heal_popup {
        let popup_y = center.y - size * 0.5 - (0.5 - timer) * 40.0;
        let popup_alpha = (timer * 2.0 * 255.0).min(255.0) as u8;
        ui.painter().text(
            egui::pos2(center.x, popup_y),
            egui::Align2::CENTER_CENTER,
            format!("+{}", amount),
            egui::FontId::proportional(size * 0.25),
            egui::Color32::from_rgba_premultiplied(80, 255, 80, popup_alpha),
        );
    }
}
