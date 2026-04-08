use shared::battle::{BattleAction, BattleResult, BattleSide, StatKind};
use shared::unit::Unit;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub struct UnitSnapshot {
    pub id: u64,
    pub name: String,
    pub base_hp: i32,
    pub base_pwr: i32,
    pub current_hp: i32,
    pub current_pwr: i32,
    pub side: BattleSide,
    pub slot: u8,
    pub alive: bool,
}

#[derive(Clone, Debug)]
pub struct BattlePlaybackState {
    pub result: BattleResult,
    pub units: HashMap<u64, Unit>,
    pub current_index: usize,
    pub auto_play: bool,
    pub speed: f32,
}

impl BattlePlaybackState {
    pub fn new(result: BattleResult, units: HashMap<u64, Unit>) -> Self {
        Self {
            result,
            units,
            current_index: 0,
            auto_play: false,
            speed: 1.0,
        }
    }

    pub fn total_actions(&self) -> usize {
        self.result.actions.len()
    }

    pub fn is_finished(&self) -> bool {
        self.current_index >= self.total_actions()
    }

    pub fn step_forward(&mut self) {
        if self.current_index < self.total_actions() {
            self.current_index += 1;
            // Skip Wait and Vfx actions
            while self.current_index < self.total_actions() {
                match &self.result.actions[self.current_index] {
                    BattleAction::Wait { .. } | BattleAction::Vfx { .. } => {
                        self.current_index += 1;
                    }
                    _ => break,
                }
            }
        }
    }

    pub fn step_back(&mut self) {
        if self.current_index > 0 {
            self.current_index -= 1;
            // Skip Wait and Vfx actions backwards
            while self.current_index > 0 {
                match &self.result.actions[self.current_index] {
                    BattleAction::Wait { .. } | BattleAction::Vfx { .. } => {
                        self.current_index -= 1;
                    }
                    _ => break,
                }
            }
        }
    }

    pub fn reset(&mut self) {
        self.current_index = 0;
        self.auto_play = false;
    }

    pub fn jump_to_end(&mut self) {
        self.current_index = self.total_actions();
    }

    /// Replay actions[0..current_index] to build current unit snapshots.
    pub fn snapshots(&self) -> HashMap<u64, UnitSnapshot> {
        let mut snaps: HashMap<u64, UnitSnapshot> = HashMap::new();

        for action in self.result.actions.iter().take(self.current_index) {
            match action {
                BattleAction::Spawn { unit, slot, side } => {
                    if let Some(u) = self.units.get(unit) {
                        snaps.insert(
                            *unit,
                            UnitSnapshot {
                                id: *unit,
                                name: u.name.clone(),
                                base_hp: u.hp,
                                base_pwr: u.pwr,
                                current_hp: u.hp,
                                current_pwr: u.pwr,
                                side: *side,
                                slot: *slot,
                                alive: true,
                            },
                        );
                    }
                }
                BattleAction::Damage { target, amount, .. } => {
                    if let Some(snap) = snaps.get_mut(target) {
                        snap.current_hp -= amount;
                    }
                }
                BattleAction::Heal { target, amount, .. } => {
                    if let Some(snap) = snaps.get_mut(target) {
                        snap.current_hp = (snap.current_hp + amount).min(snap.base_hp);
                    }
                }
                BattleAction::Death { unit } => {
                    if let Some(snap) = snaps.get_mut(unit) {
                        snap.alive = false;
                        snap.current_hp = 0;
                    }
                }
                BattleAction::StatChange { unit, stat, delta } => {
                    if let Some(snap) = snaps.get_mut(unit) {
                        match stat {
                            StatKind::Hp => snap.current_hp += delta,
                            StatKind::Pwr => snap.current_pwr += delta,
                            StatKind::Dmg => {} // damage tracking not shown
                        }
                    }
                }
                _ => {}
            }
        }

        snaps
    }

    /// Get visible actions (everything except Wait/Vfx) up to current_index.
    pub fn visible_actions(&self) -> Vec<(usize, &BattleAction)> {
        self.result
            .actions
            .iter()
            .take(self.current_index)
            .enumerate()
            .filter(|(_, a)| !matches!(a, BattleAction::Wait { .. } | BattleAction::Vfx { .. }))
            .collect()
    }

    pub fn unit_name(&self, id: u64) -> String {
        self.units
            .get(&id)
            .map(|u| u.name.clone())
            .unwrap_or_else(|| format!("Unit #{id}"))
    }

    /// Get teams as (left, right) sorted by slot.
    pub fn teams(&self, snaps: &HashMap<u64, UnitSnapshot>) -> (Vec<UnitSnapshot>, Vec<UnitSnapshot>) {
        let mut left: Vec<_> = snaps
            .values()
            .filter(|s| s.side == BattleSide::Left)
            .cloned()
            .collect();
        let mut right: Vec<_> = snaps
            .values()
            .filter(|s| s.side == BattleSide::Right)
            .cloned()
            .collect();
        left.sort_by_key(|s| s.slot);
        right.sort_by_key(|s| s.slot);
        (left, right)
    }

    /// Get the first alive unit on each side (the active fighters).
    pub fn active_fighters(&self, snaps: &HashMap<u64, UnitSnapshot>) -> (Option<u64>, Option<u64>) {
        let (left, right) = self.teams(snaps);
        let left_active = left.iter().find(|s| s.alive).map(|s| s.id);
        let right_active = right.iter().find(|s| s.alive).map(|s| s.id);
        (left_active, right_active)
    }
}
