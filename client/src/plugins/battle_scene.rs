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

// ===== Turn-Based Structure =====

/// A single event within a turn (one ability use + its effects).
#[derive(Clone, Debug)]
struct TurnEvent {
    source_name: String,
    source_id: u64,
    ability_name: String,
    effects: Vec<(String, egui::Color32)>, // description + color
    actions: Vec<BattleAction>,            // raw actions for state application
}

/// A full turn card: one round of the battle.
#[derive(Clone, Debug)]
struct TurnCard {
    turn_number: usize,
    events: Vec<TurnEvent>,
}

/// Parse flat action list into structured turn cards.
fn build_turns(actions: &[BattleAction], units: &[BattleUnitVisual]) -> Vec<TurnCard> {
    let mut turns: Vec<TurnCard> = Vec::new();
    let mut current_turn = TurnCard {
        turn_number: 1,
        events: Vec::new(),
    };
    let mut current_event: Option<TurnEvent> = None;
    let mut turn_num = 1;

    let name = |id: u64| -> String {
        units
            .iter()
            .find(|u| u.id == id)
            .map(|u| u.name.clone())
            .unwrap_or_else(|| format!("#{}", id))
    };

    for action in actions {
        match action {
            BattleAction::AbilityUsed {
                source,
                ability_name,
            } => {
                // Flush previous event
                if let Some(ev) = current_event.take() {
                    current_turn.events.push(ev);
                }
                current_event = Some(TurnEvent {
                    source_name: name(*source),
                    source_id: *source,
                    ability_name: ability_name.clone(),
                    effects: Vec::new(),
                    actions: Vec::new(),
                });
            }
            BattleAction::Damage {
                source,
                target,
                amount,
            } => {
                let desc = format!(
                    "{} deals {} dmg to {}",
                    name(*source),
                    amount,
                    name(*target)
                );
                if let Some(ref mut ev) = current_event {
                    ev.effects
                        .push((desc, egui::Color32::from_rgb(255, 120, 120)));
                    ev.actions.push(action.clone());
                }
            }
            BattleAction::Heal {
                source,
                target,
                amount,
            } => {
                let desc = format!("{} heals {} for {}", name(*source), amount, name(*target));
                if let Some(ref mut ev) = current_event {
                    ev.effects
                        .push((desc, egui::Color32::from_rgb(120, 255, 120)));
                    ev.actions.push(action.clone());
                }
            }
            BattleAction::StatChange { unit, stat, delta } => {
                let desc = format!("{} {:?} {:+}", name(*unit), stat, delta);
                if let Some(ref mut ev) = current_event {
                    ev.effects
                        .push((desc, egui::Color32::from_rgb(200, 200, 100)));
                    ev.actions.push(action.clone());
                }
            }
            BattleAction::Death { unit: uid } => {
                let desc = format!("{} dies!", name(*uid));
                if let Some(ref mut ev) = current_event {
                    ev.effects
                        .push((desc, egui::Color32::from_rgb(255, 50, 50)));
                    ev.actions.push(action.clone());
                } else {
                    // Death outside an event — create standalone
                    let mut ev = TurnEvent {
                        source_name: name(*uid),
                        source_id: *uid,
                        ability_name: "Death".to_string(),
                        effects: vec![(
                            format!("{} dies!", name(*uid)),
                            egui::Color32::from_rgb(255, 50, 50),
                        )],
                        actions: vec![action.clone()],
                    };
                    current_turn.events.push(ev);
                }
            }
            BattleAction::Fatigue { amount } => {
                // Fatigue = new turn
                if let Some(ev) = current_event.take() {
                    current_turn.events.push(ev);
                }
                if !current_turn.events.is_empty() {
                    turns.push(current_turn.clone());
                    turn_num += 1;
                }
                current_turn = TurnCard {
                    turn_number: turn_num,
                    events: Vec::new(),
                };
                current_turn.events.push(TurnEvent {
                    source_name: "Fatigue".to_string(),
                    source_id: 0,
                    ability_name: "Fatigue".to_string(),
                    effects: vec![(
                        format!("All units take {} dmg", amount),
                        egui::Color32::from_rgb(255, 200, 50),
                    )],
                    actions: vec![action.clone()],
                });
            }
            BattleAction::Spawn { .. } => {
                // Skip spawns
            }
            _ => {}
        }
    }

    // Flush remaining
    if let Some(ev) = current_event {
        current_turn.events.push(ev);
    }
    if !current_turn.events.is_empty() {
        turns.push(current_turn);
    }

    // Split into per-turn cards: every 2-4 events form a turn
    // (group by natural pauses between BeforeStrike rounds)
    let mut result = Vec::new();
    let mut card = TurnCard {
        turn_number: 1,
        events: Vec::new(),
    };
    let mut t = 1;
    for turn in turns {
        for event in turn.events {
            card.events.push(event);
            // Start new card every ~4 events or on fatigue
            if card.events.len() >= 4
                || card
                    .events
                    .last()
                    .map(|e| e.ability_name == "Fatigue")
                    .unwrap_or(false)
            {
                card.turn_number = t;
                result.push(card);
                t += 1;
                card = TurnCard {
                    turn_number: t,
                    events: Vec::new(),
                };
            }
        }
    }
    if !card.events.is_empty() {
        card.turn_number = t;
        result.push(card);
    }

    result
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
    pub turns: Vec<TurnCard>,
    pub current_turn: usize,
    pub current_event: usize,
    pub event_progress: f32, // 0.0 → 1.0 per event (1 second each)
    pub playing: bool,
    pub speed: f32,
    pub completed_turns: Vec<TurnCard>, // for log
}

impl BattleSceneState {
    pub fn load(&mut self, result: BattleResult, units: Vec<BattleUnitVisual>) {
        let turns = build_turns(&result.actions, &units);
        self.initial_units = units.clone();
        self.units = units;
        self.turns = turns;
        self.result = Some(result);
        self.current_turn = 0;
        self.current_event = 0;
        self.event_progress = 0.0;
        self.playing = false;
        self.speed = 1.0;
        self.completed_turns = Vec::new();
    }

    /// Rebuild unit state by replaying all actions up to current turn/event.
    fn rebuild_units(&mut self) {
        self.units = self.initial_units.clone();
        self.completed_turns.clear();

        for t in 0..self.current_turn.min(self.turns.len()) {
            for event in &self.turns[t].events {
                for action in &event.actions {
                    apply_action(&mut self.units, action);
                }
            }
            self.completed_turns.push(self.turns[t].clone());
        }
        // Apply events in current turn up to current_event
        if self.current_turn < self.turns.len() {
            let turn = &self.turns[self.current_turn];
            for e in 0..self.current_event.min(turn.events.len()) {
                for action in &turn.events[e].actions {
                    apply_action(&mut self.units, action);
                }
            }
        }
    }

    fn total_events(&self) -> usize {
        self.turns.iter().map(|t| t.events.len()).sum()
    }

    fn active_event(&self) -> Option<&TurnEvent> {
        self.turns
            .get(self.current_turn)?
            .events
            .get(self.current_event)
    }

    fn active_turn(&self) -> Option<&TurnCard> {
        self.turns.get(self.current_turn)
    }
}

fn apply_action(units: &mut [BattleUnitVisual], action: &BattleAction) {
    match action {
        BattleAction::Damage { target, amount, .. } => {
            if let Some(u) = units.iter_mut().find(|u| u.id == *target) {
                u.dmg += amount;
            }
        }
        BattleAction::Heal { target, amount, .. } => {
            if let Some(u) = units.iter_mut().find(|u| u.id == *target) {
                u.dmg = (u.dmg - amount).max(0);
            }
        }
        BattleAction::Death { unit } => {
            if let Some(u) = units.iter_mut().find(|u| u.id == *unit) {
                u.alive = false;
            }
        }
        BattleAction::StatChange { unit, stat, delta } => {
            if let Some(u) = units.iter_mut().find(|u| u.id == *unit) {
                match stat {
                    shared::battle::StatKind::Pwr => u.pwr = (u.pwr + delta).max(0),
                    shared::battle::StatKind::Hp => u.hp = (u.hp + delta).max(0),
                    _ => {}
                }
            }
        }
        BattleAction::Fatigue { amount } => {
            for u in units.iter_mut().filter(|u| u.alive) {
                u.dmg += amount;
            }
        }
        _ => {}
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

    let winner = state.result.as_ref().unwrap().winner;
    let result_turns = state.result.as_ref().unwrap().turns;
    let total_turn_cards = state.turns.len();
    let is_done = state.current_turn >= total_turn_cards;

    // Space key
    if ctx.input(|i| i.key_pressed(egui::Key::Space)) {
        state.playing = !state.playing;
    }

    // Advance
    if state.playing && !is_done {
        state.event_progress += time.delta_secs() * state.speed;
        if state.event_progress >= 1.0 {
            state.event_progress = 0.0;
            let ct = state.current_turn;
            let ce = state.current_event;
            let turn_len = state.turns.get(ct).map(|t| t.events.len()).unwrap_or(0);

            if ce < turn_len {
                let actions: Vec<BattleAction> = state.turns[ct].events[ce].actions.clone();
                for action in &actions {
                    apply_action(&mut state.units, action);
                }
                state.current_event += 1;
            }

            if state.current_event >= turn_len && turn_len > 0 {
                let completed = state.turns[ct].clone();
                state.completed_turns.push(completed);
                state.current_turn += 1;
                state.current_event = 0;
            }
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
            if is_done {
                ui.colored_label(color, label);
            }
            ui.label(format!(
                "Turn {}/{}",
                state.current_turn + 1,
                total_turn_cards
            ));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Exit").clicked() {
                    state.result = None;
                    next_state.set(GameState::Home);
                }
            });
        });
    });

    // === RIGHT: Battle Log ===
    egui::SidePanel::right("battle_log")
        .resizable(true)
        .default_width(280.0)
        .min_width(150.0)
        .show(ctx, |ui| {
            ui.heading("Battle Log");
            ui.separator();
            egui::ScrollArea::vertical()
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for turn in &state.completed_turns {
                        ui.group(|ui| {
                            ui.label(
                                egui::RichText::new(format!("Turn {}", turn.turn_number)).strong(),
                            );
                            for event in &turn.events {
                                ui.horizontal(|ui| {
                                    ui.colored_label(
                                        egui::Color32::from_rgb(100, 170, 255),
                                        format!("{}: {}", event.source_name, event.ability_name),
                                    );
                                });
                                for (desc, color) in &event.effects {
                                    ui.colored_label(*color, format!("  {}", desc));
                                }
                            }
                        });
                    }
                    if state.completed_turns.is_empty() && !is_done {
                        ui.colored_label(egui::Color32::GRAY, "Press Space or Play to start...");
                    }
                });
        });

    // === BOTTOM: Controls ===
    egui::TopBottomPanel::bottom("battle_controls")
        .exact_height(40.0)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("⏮").clicked() {
                    state.current_turn = 0;
                    state.current_event = 0;
                    state.event_progress = 0.0;
                    state.rebuild_units();
                }
                if state.playing {
                    if ui.button("⏸ Pause").clicked() {
                        state.playing = false;
                    }
                } else if ui.button("▶ Play").clicked() {
                    if is_done {
                        state.current_turn = 0;
                        state.current_event = 0;
                        state.event_progress = 0.0;
                        state.rebuild_units();
                    }
                    state.playing = true;
                }
                if ui.button("⏭").clicked() {
                    state.current_turn = total_turn_cards;
                    state.current_event = 0;
                    state.rebuild_units();
                }
                ui.separator();
                ui.label("Speed:");
                for &s in &[0.5f32, 1.0, 2.0, 4.0] {
                    if ui
                        .selectable_label((state.speed - s).abs() < 0.01, format!("x{}", s))
                        .clicked()
                    {
                        state.speed = s;
                    }
                }
            });
        });

    // Pre-extract data for central panel (avoids borrow conflicts)
    let active_source_id = state.active_event().map(|e| e.source_id).unwrap_or(0);
    let active_turn_clone = state.active_turn().cloned();
    let current_event_idx = state.current_event;
    let event_progress = state.event_progress;
    let units_clone: Vec<BattleUnitVisual> = state.units.clone();

    // === CENTRAL: Battle Area ===
    egui::CentralPanel::default().show(ctx, |ui| {
        let avail = ui.available_rect_before_wrap();
        let painter = ui.painter().clone();

        painter.rect_filled(avail, 0.0, egui::Color32::from_rgb(12, 12, 20));

        let row_h = (avail.height() * 0.25).min(100.0);
        let unit_size = (row_h * 0.8).min(75.0);
        let top_y = avail.top() + row_h * 0.5 + 10.0;
        let bottom_y = avail.bottom() - row_h * 0.5 - 10.0;
        let mid_y = (top_y + bottom_y) * 0.5;

        let mut unit_positions: std::collections::HashMap<u64, egui::Pos2> =
            std::collections::HashMap::new();

        let left: Vec<&BattleUnitVisual> = units_clone
            .iter()
            .filter(|u| u.side == BattleSide::Left)
            .collect();
        let right: Vec<&BattleUnitVisual> = units_clone
            .iter()
            .filter(|u| u.side == BattleSide::Right)
            .collect();

        // Draw left team
        for (i, unit) in left.iter().enumerate() {
            let x = avail.left() + (i as f32 + 0.5) * avail.width() / left.len().max(1) as f32;
            let pos = egui::pos2(x, top_y);
            unit_positions.insert(unit.id, pos);

            let alpha = if !unit.alive { 60 } else { 255 };
            let mut c = egui::Color32::from_rgba_premultiplied(
                unit.color.r(),
                unit.color.g(),
                unit.color.b(),
                alpha,
            );
            let sz = if !unit.alive {
                unit_size * 0.6
            } else {
                unit_size
            };

            // Highlight active unit
            let is_active = unit.id == active_source_id && !is_done;
            if is_active {
                painter.circle_stroke(pos, sz * 0.6, egui::Stroke::new(3.0, egui::Color32::YELLOW));
            }

            let rect = egui::Rect::from_center_size(pos, egui::vec2(sz, sz));
            paint_default_unit(rect, c, unit.hp, unit.pwr, unit.dmg, &unit.name, ui);
        }

        // Draw right team
        for (i, unit) in right.iter().enumerate() {
            let x = avail.left() + (i as f32 + 0.5) * avail.width() / right.len().max(1) as f32;
            let pos = egui::pos2(x, bottom_y);
            unit_positions.insert(unit.id, pos);

            let alpha = if !unit.alive { 60 } else { 255 };
            let c = egui::Color32::from_rgba_premultiplied(
                unit.color.r(),
                unit.color.g(),
                unit.color.b(),
                alpha,
            );
            let sz = if !unit.alive {
                unit_size * 0.6
            } else {
                unit_size
            };

            let is_active = unit.id == active_source_id && !is_done;
            if is_active {
                painter.circle_stroke(pos, sz * 0.6, egui::Stroke::new(3.0, egui::Color32::YELLOW));
            }

            let rect = egui::Rect::from_center_size(pos, egui::vec2(sz, sz));
            paint_default_unit(rect, c, unit.hp, unit.pwr, unit.dmg, &unit.name, ui);
        }

        // Team labels
        painter.text(
            egui::pos2(avail.left() + 8.0, avail.top() + 4.0),
            egui::Align2::LEFT_TOP,
            "Left Team",
            egui::FontId::proportional(12.0),
            egui::Color32::from_rgb(100, 100, 100),
        );
        painter.text(
            egui::pos2(avail.left() + 8.0, bottom_y - unit_size * 0.5 - 14.0),
            egui::Align2::LEFT_TOP,
            "Right Team",
            egui::FontId::proportional(12.0),
            egui::Color32::from_rgb(100, 100, 100),
        );

        // === TURN CARD in center ===
        if let Some(turn) = &active_turn_clone {
            let card_w = 350.0f32.min(avail.width() * 0.6);
            let card_h = (turn.events.len() as f32 * 40.0 + 30.0).min(avail.height() * 0.4);
            let card_rect = egui::Rect::from_center_size(
                egui::pos2(avail.center().x, mid_y),
                egui::vec2(card_w, card_h),
            );

            // Card background
            painter.rect_filled(
                card_rect,
                10.0,
                egui::Color32::from_rgba_premultiplied(15, 15, 28, 230),
            );
            painter.rect_stroke(
                card_rect,
                10.0,
                egui::Stroke::new(2.0, egui::Color32::from_rgb(60, 60, 80)),
                egui::StrokeKind::Outside,
            );

            // Render events
            let mut y = card_rect.top() + 8.0;
            for (i, event) in turn.events.iter().enumerate() {
                let is_current = i == current_event_idx;
                let is_past = i < current_event_idx;
                let alpha = if is_past {
                    120u8
                } else if is_current {
                    255
                } else {
                    40
                };

                // Ability header
                let header_color = egui::Color32::from_rgba_premultiplied(100, 170, 255, alpha);
                painter.text(
                    egui::pos2(card_rect.left() + 12.0, y),
                    egui::Align2::LEFT_TOP,
                    format!("{}: {}", event.source_name, event.ability_name),
                    egui::FontId::proportional(13.0),
                    header_color,
                );
                y += 16.0;

                // Effects
                for (desc, color) in &event.effects {
                    let c = egui::Color32::from_rgba_premultiplied(
                        color.r(),
                        color.g(),
                        color.b(),
                        alpha,
                    );
                    painter.text(
                        egui::pos2(card_rect.left() + 20.0, y),
                        egui::Align2::LEFT_TOP,
                        desc,
                        egui::FontId::proportional(11.0),
                        c,
                    );
                    y += 14.0;
                }
                y += 4.0;

                // Draw line for current event's damage/heal
                if is_current && event_progress > 0.2 {
                    for action in &event.actions {
                        match action {
                            BattleAction::Damage { source, target, .. } => {
                                if let (Some(&from), Some(&to)) =
                                    (unit_positions.get(source), unit_positions.get(target))
                                {
                                    let progress = ((event_progress - 0.2) / 0.5).min(1.0);
                                    let current_to = egui::pos2(
                                        from.x + (to.x - from.x) * progress,
                                        from.y + (to.y - from.y) * progress,
                                    );
                                    painter.line_segment(
                                        [from, current_to],
                                        egui::Stroke::new(
                                            3.0,
                                            egui::Color32::from_rgb(255, 80, 80),
                                        ),
                                    );
                                    // Glow
                                    painter.line_segment(
                                        [from, current_to],
                                        egui::Stroke::new(
                                            8.0,
                                            egui::Color32::from_rgba_premultiplied(255, 80, 80, 60),
                                        ),
                                    );
                                    // Arrowhead
                                    if progress > 0.2 {
                                        let dir = egui::vec2(to.x - from.x, to.y - from.y);
                                        let len = dir.length();
                                        if len > 0.0 {
                                            let norm = dir / len;
                                            let perp = egui::vec2(-norm.y, norm.x);
                                            let tip = current_to;
                                            painter.line_segment(
                                                [
                                                    egui::pos2(
                                                        tip.x - norm.x * 10.0 + perp.x * 5.0,
                                                        tip.y - norm.y * 10.0 + perp.y * 5.0,
                                                    ),
                                                    tip,
                                                ],
                                                egui::Stroke::new(
                                                    3.0,
                                                    egui::Color32::from_rgb(255, 80, 80),
                                                ),
                                            );
                                            painter.line_segment(
                                                [
                                                    egui::pos2(
                                                        tip.x - norm.x * 10.0 - perp.x * 5.0,
                                                        tip.y - norm.y * 10.0 - perp.y * 5.0,
                                                    ),
                                                    tip,
                                                ],
                                                egui::Stroke::new(
                                                    3.0,
                                                    egui::Color32::from_rgb(255, 80, 80),
                                                ),
                                            );
                                        }
                                    }
                                }
                            }
                            BattleAction::Heal { source, target, .. } => {
                                if let (Some(&from), Some(&to)) =
                                    (unit_positions.get(source), unit_positions.get(target))
                                {
                                    let progress = ((event_progress - 0.2) / 0.5).min(1.0);
                                    let current_to = egui::pos2(
                                        from.x + (to.x - from.x) * progress,
                                        from.y + (to.y - from.y) * progress,
                                    );
                                    painter.line_segment(
                                        [from, current_to],
                                        egui::Stroke::new(
                                            3.0,
                                            egui::Color32::from_rgb(80, 230, 80),
                                        ),
                                    );
                                    painter.line_segment(
                                        [from, current_to],
                                        egui::Stroke::new(
                                            8.0,
                                            egui::Color32::from_rgba_premultiplied(80, 230, 80, 60),
                                        ),
                                    );
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        // Show victory card when done
        if is_done {
            let (text, color) = match winner {
                BattleSide::Left => ("LEFT TEAM WINS!", egui::Color32::from_rgb(100, 255, 100)),
                BattleSide::Right => ("RIGHT TEAM WINS!", egui::Color32::from_rgb(255, 100, 100)),
            };
            let card_rect = egui::Rect::from_center_size(
                egui::pos2(avail.center().x, mid_y),
                egui::vec2(300.0, 60.0),
            );
            painter.rect_filled(
                card_rect,
                10.0,
                egui::Color32::from_rgba_premultiplied(15, 15, 28, 230),
            );
            painter.rect_stroke(
                card_rect,
                10.0,
                egui::Stroke::new(2.0, color),
                egui::StrokeKind::Outside,
            );
            painter.text(
                card_rect.center(),
                egui::Align2::CENTER_CENTER,
                text,
                egui::FontId::proportional(24.0),
                color,
            );
        }
    });
}
