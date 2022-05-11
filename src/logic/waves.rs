use super::*;

impl Logic<'_> {
    pub fn check_next_wave(&mut self) {
        if self
            .model
            .units
            .iter()
            .filter(|unit| unit.faction != Faction::Player)
            .count()
            == 0
            && self.model.spawning_units.is_empty()
            && self.model.time_bombs.is_empty()
            && self.effects.is_empty()
        {
            if self.model.config.waves.is_empty() {
                // End of round -> go to shop
                self.model.transition = true;
            } else {
                // Next wave
                let wave = self.model.config.waves.remove(0);
                for (spawn_point, units) in wave {
                    let spawn_point = self.model.config.spawn_points[&spawn_point];
                    for unit_type in units {
                        self.spawn_unit(&unit_type, Faction::Enemy, spawn_point);
                    }
                }
            }
        }
    }
}
