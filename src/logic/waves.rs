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
            if !self.config.waves.is_empty() {
                let wave = self.config.waves.remove(0);
                for (spawn_point, units) in wave {
                    let spawn_point = self.config.spawn_points[&spawn_point];
                    for unit_type in units {
                        let template = self.assets.units.map[&unit_type].clone();
                        self.spawn_unit(
                            &template,
                            Faction::Enemy,
                            spawn_point
                                + vec2(
                                    global_rng().gen_range(Coord::new(-1.0)..=Coord::new(1.0)),
                                    global_rng().gen_range(Coord::new(-1.0)..=Coord::new(1.0)),
                                ) * Coord::new(0.01),
                        );
                    }
                }
            }
        }
    }
}
