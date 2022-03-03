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
    pub fn update(&mut self, delta_time: Time) {
        let ids: Vec<Id> = self.units.ids().copied().collect();
        for unit_id in ids {
            let mut unit = self.units.remove(&unit_id).unwrap();
            self.process_movement(&mut unit, delta_time);
            self.process_statuses(&mut unit, delta_time);
            self.process_collisions(&mut unit);
            self.process_targeting(&mut unit);
            self.process_attacks(&mut unit, delta_time);
            self.process_cooldowns(&mut unit, delta_time);
            self.units.insert(unit);
        }
        self.process_projectiles(delta_time);
        self.process_deaths();
        self.check_next_wave();
    }
}
