use super::*;

mod abilities;
mod alliances;
mod attacks;
mod collisions;
mod deaths;
mod effects;
mod movement;
mod projectiles;
mod spawn;
mod statuses;
mod targeting;
mod time_bombs;
mod util;
mod waves;

pub use effects::*;
pub use util::*;

pub struct Logic<'a> {
    pub model: &'a mut Model,
    pub delta_time: Time,
    pub effects: Vec<QueuedEffect<Effect>>,
    pub pressed_keys: Vec<Key>,
    pub render: Option<&'a mut Render>,
}

impl<'a> Logic<'a> {
    pub fn initialize(model: &'a mut Model, config: &Config) {
        let mut logic = Self {
            model,
            delta_time: Time::new(0.0),
            effects: Vec::new(),
            pressed_keys: Vec::new(),
            render: None,
        };
        for unit_type in &config.player {
            logic.spawn_unit(unit_type, Faction::Player, Vec2::ZERO);
        }
        logic.process();
    }
    pub fn process(&mut self) {
        self.process_time_bombs();
        self.process_spawns();
        self.process_abilities();
        self.process_movement();
        self.process_statuses();
        self.process_collisions();
        self.process_targeting();
        self.process_attacks();
        self.process_cooldowns();
        self.process_projectiles();
        self.process_deaths();
        self.check_next_wave();
        self.process_effects();
    }
    fn process_units(&mut self, mut f: impl FnMut(&mut Self, &mut Unit)) {
        let ids: Vec<Id> = self.model.units.ids().copied().collect();
        for id in ids {
            let mut unit = self.model.units.remove(&id).unwrap();
            f(self, &mut unit);
            self.model.units.insert(unit);
        }
    }
}

impl Game {
    pub fn update(&mut self, delta_time: Time) {
        let mut logic = Logic {
            model: &mut self.model,
            delta_time,
            effects: Vec::new(),
            pressed_keys: mem::take(&mut self.pressed_keys),
            render: Some(&mut self.render),
        };
        logic.process();
    }
}
