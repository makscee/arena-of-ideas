use super::*;

mod attacks;
mod collisions;
mod damage;
mod deaths;
mod effects;
mod movement;
mod projectiles;
mod spawn;
mod statuses;
mod targeting;
mod util;
mod waves;

pub use util::*;

impl Game {
    fn process_units(&mut self, mut f: impl FnMut(&mut Self, &mut Unit)) {
        let ids: Vec<Id> = self.units.ids().copied().collect();
        for id in ids {
            let mut unit = self.units.remove(&id).unwrap();
            f(self, &mut unit);
            self.units.insert(unit);
        }
    }
    pub fn update(&mut self, delta_time: Time) {
        self.delta_time = delta_time;
        self.process_spawns();
        self.process_movement();
        self.process_statuses();
        self.process_collisions();
        self.process_targeting();
        self.process_attacks();
        self.process_cooldowns();
        self.process_projectiles();
        self.process_deaths();
        self.check_next_wave();
    }
}
