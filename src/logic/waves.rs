use super::*;

impl Game {
    pub fn check_next_wave(&mut self) {
        if self
            .units
            .iter()
            .filter(|unit| unit.faction != Faction::Player)
            .count()
            == 0
        {
            if !self.assets.config.waves.is_empty() {
                let wave = self.assets.config.waves.remove(0);
                for (spawn_point, units) in wave {
                    let spawn_point = self.assets.config.spawn_points[&spawn_point];
                    for unit_type in units {
                        self.spawn_unit(&unit_type, Faction::Enemy, spawn_point);
                    }
                }
            }
        }
    }
}
