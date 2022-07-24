use std::collections::VecDeque;

use super::*;

mod abilities;
mod actions;
mod deaths;
mod effects;
mod events;
mod particles;
mod round;
mod spawn;
mod statuses;
mod targeting;
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
        self.init_player(&self.model.config.clone());
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
        while self.model.current_tick.tick_time >= Time::new(TICK_TIME) {
            self.tick();
        }
        self.process_particles();
        self.process_statuses();
        self.process_spawns();
        self.process_abilities();
        self.process_targeting();
        self.process_actions();
        self.process_render_positions();
        self.process_round();
        self.process_effects();
        self.process_deaths();
        self.process_time();
        self.model.render_model.update(self.delta_time.as_f32())
    }
    fn tick(&mut self) {
        // TODO: check if some actions did not perform in time
        self.model.current_tick = TickModel::new();
        self.tick_cooldowns();
        self.model.current_tick_num += 1;
    }
    fn process_units(&mut self, mut f: impl FnMut(&mut Self, &mut Unit)) {
        let ids: Vec<Id> = self.model.units.ids().copied().collect();
        for id in ids {
            let mut unit = self.model.units.remove(&id).unwrap();
            f(self, &mut unit);
            self.model.units.insert(unit);
        }
    }
    fn init_player(&mut self, config: &Config) {
        let mut to_spawn = config
            .player
            .iter()
            .map(|unit| (unit, self.model.unit_templates[unit].clone()))
            .collect::<Vec<_>>();

        for unit_type in &config.player {
            self.spawn_unit(unit_type, Faction::Player, Position::zero(Faction::Player));
        }
    }
}
