use super::*;

impl Logic<'_> {
    pub fn wave_update(&mut self) {
        // if self
        //     .model
        //     .units
        //     .iter()
        //     .filter(|unit| unit.faction != Faction::Player)
        //     .count()
        //     == 0
        //     && self.model.spawning_units.is_empty()
        //     && self.model.time_bombs.is_empty()
        //     && self.effects.is_empty()
        // {
        // if self.model.config.waves.is_empty() {
        //     // End of round -> go to shop
        //     self.model.transition = true;
        // } else {
        //     // Next wave
        //     let wave = self.model.config.waves.remove(0);
        //     for (spawn_point, units) in wave {
        //         let spawn_point = self.model.config.spawn_points[&spawn_point];
        //         for unit_type in units {
        //             self.spawn_unit(&unit_type, Faction::Enemy, spawn_point);
        //         }
        //     }
        // }
        // }

        let wait_clear = self
            .model
            .rounds
            .front()
            .and_then(|round| round.waves.front())
            .map(|wave| wave.wait_clear)
            .unwrap_or(false);
        if wait_clear
            && self
                .model
                .units
                .iter()
                .any(|unit| unit.faction != Faction::Player)
        {
            return;
        }

        if self.model.wave_delay > Time::ZERO {
            self.model.wave_delay -= self.delta_time;
            return;
        }

        if let Some(current) = self.model.rounds.front_mut() {
            if let Some(wave) = current.waves.front_mut() {
                let mut next_spawns = wave
                    .spawns
                    .iter_mut()
                    .filter_map(|(point, spawns)| match spawns.front_mut() {
                        Some(spawn) => {
                            let point = point.clone();
                            spawn.count -= 1;
                            if spawn.count == 0 {
                                Some((point, spawns.pop_front().unwrap().r#type))
                            } else {
                                Some((point, spawn.r#type.clone()))
                            }
                        }
                        None => None,
                    })
                    .collect::<Vec<_>>();
                if !next_spawns.is_empty() {
                    // Continue spawning wave
                    self.model.wave_delay = wave.between_delay;
                    for (point, unit_type) in next_spawns {
                        let position = *self
                            .model
                            .config
                            .spawn_points
                            .get(&point)
                            .expect(&format!("Failed to find spawnpoint: {point}"));
                        self.spawn_unit(&unit_type, Faction::Enemy, position);
                    }
                } else {
                    // Next wave
                    current.waves.pop_front();
                    if let Some(next_wave) = current.waves.front() {
                        self.model.wave_delay = next_wave.start_delay;
                    }
                }
            }
        }
    }
}
