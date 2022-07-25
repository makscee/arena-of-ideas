use std::collections::VecDeque;

use super::*;

mod abilities;
mod actions;
mod deaths;
mod effects;
mod events;
mod particles;
mod spawn;
mod statuses;
mod targeting;
mod tick;
mod time;
mod util;

pub use effects::*;
pub use events::*;
pub use util::*;

pub struct Logic {
    pub model: Model,
    pub delta_time: Time,
    pub effects: VecDeque<QueuedEffect<Effect>>,
    pub paused: bool,
}

impl Logic {
    pub fn initialize(&mut self, events: &mut Events) {
        self.init_player(self.model.config.player.clone());
        self.init_enemies();
        self.init_time(events);
        self.init_abilities(events);
    }

    pub fn new(mut model: Model) -> Self {
        Self {
            model,
            delta_time: Time::new(0.0),
            effects: VecDeque::new(),
            paused: false,
        }
    }

    pub fn update(&mut self, delta_time: f64) {
        self.delta_time = Time::new(delta_time as f32);
        self.process_tick();
        self.process_particles();
        self.process_statuses();
        self.process_spawns();
        self.process_abilities();
        self.process_targeting();
        self.process_actions();
        self.process_render_positions();
        self.process_effects();
        self.process_deaths();
        self.process_time();
        self.model.render_model.update(self.delta_time.as_f32())
    }
    fn process_units(&mut self, mut f: impl FnMut(&mut Self, &mut Unit)) {
        let ids: Vec<Id> = self.model.units.ids().copied().collect();
        for id in ids {
            let mut unit = self.model.units.remove(&id).unwrap();
            f(self, &mut unit);
            self.model.units.insert(unit);
        }
    }
    fn process_units_random(&mut self, mut f: impl FnMut(&mut Self, &mut Unit)) {
        let mut ids: Vec<Id> = self.model.units.ids().copied().collect();
        ids.shuffle(&mut global_rng());
        for id in ids {
            let mut unit = self.model.units.remove(&id).unwrap();
            f(self, &mut unit);
            self.model.units.insert(unit);
        }
    }
    fn init_player(&mut self, player: Vec<UnitType>) {
        for unit_type in &player {
            self.spawn_unit(unit_type, Faction::Player, Position::zero(Faction::Player));
        }
    }
    fn init_enemies(&mut self) {
        let round = self.model.round.clone();
        for unit_type in &round.enemies {
            let unit = self.spawn_unit(&unit_type, Faction::Enemy, Position::zero(Faction::Enemy));
            let unit = self.model.units.get_mut(&unit).unwrap();
            let statuses = round.statuses.iter().map(|status| {
                status.get(&self.model.statuses).clone().attach(
                    Some(unit.id),
                    None,
                    &mut self.model.next_id,
                )
            });
            unit.all_statuses.extend(statuses);
        }
    }
    fn process_render_positions(&mut self) {
        self.process_units(Self::process_unit_render_positions);
    }
    fn process_unit_render_positions(&mut self, unit: &mut Unit) {
        unit.render_position +=
            (unit.position.to_world() - unit.render_position) * self.delta_time * r32(5.0);
    }
}
