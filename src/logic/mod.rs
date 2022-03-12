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
        logic.spawn_player(config);
        logic.process();
    }
    pub fn process(&mut self) {
        self.process_statuses();
        self.process_time_bombs();
        self.process_spawns();
        self.process_abilities();
        self.process_movement();
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
    fn spawn_player(&mut self, config: &Config) {
        let mut to_spawn = config
            .player
            .iter()
            .map(|unit| (unit, self.model.unit_templates[unit].clone()))
            .collect::<Vec<_>>();

        // Count members in each alliance
        let mut members = HashMap::new();
        for alliance in to_spawn
            .iter()
            .flat_map(|(_, unit)| &unit.alliances)
            .map(|alliance| *alliance as u32)
        {
            let entry = members.entry(alliance).or_insert(0);
            *entry += 1;
        }

        // Apply effects
        for (_, unit) in &mut to_spawn {
            for alliance in unit.alliances.clone() {
                let members = members[&(alliance as u32)];
                alliance.apply(unit, members);
            }
        }

        // Spawn
        for (unit_type, template) in to_spawn {
            self.spawn_template(unit_type, template, Faction::Player, Vec2::ZERO);
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
