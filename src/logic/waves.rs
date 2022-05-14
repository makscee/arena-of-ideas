use super::*;

impl Logic<'_> {
    pub fn wave_update(&mut self) {
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

        if let Some(round) = self.model.rounds.front_mut() {
            if let Some(wave) = round.waves.front_mut() {
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
                        let unit = self.spawn_unit(&unit_type, Faction::Enemy, position);
                        let unit = self.model.spawning_units.get_mut(&unit).unwrap();
                        let round = self.model.rounds.front().unwrap();
                        let statuses = round
                            .statuses
                            .iter()
                            .chain(round.waves.front().unwrap().statuses.iter());
                        for status in statuses {
                            unit.attached_statuses.push(AttachedStatus {
                                status: status.clone(),
                                caster: None,
                                time: None,
                                duration: None,
                            })
                        }
                    }
                } else {
                    // Next wave
                    round.waves.pop_front();
                    if let Some(next_wave) = round.waves.front() {
                        self.model.wave_delay = next_wave.start_delay;
                    }
                }
            }
        }
    }
}
