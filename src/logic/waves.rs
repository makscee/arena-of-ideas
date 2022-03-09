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
        {
            if !self.model.config.waves.is_empty() {
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
