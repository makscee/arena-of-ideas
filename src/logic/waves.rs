use super::*;

impl Logic<'_> {
    pub fn wave_update(&mut self) {
        let wait_clear = self
            .model
            .round
            .waves
            .front()
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

        let round = &mut self.model.round;
        if let Some(wave) = round.waves.front_mut() {
            let mut next_spawns = match wave.spawns.front_mut() {
                Some(spawn) => {
                    spawn.count -= 1;
                    if spawn.count == 0 {
                        Some(wave.spawns.pop_front().unwrap().r#type)
                    } else {
                        Some(spawn.r#type.clone())
                    }
                }
                None => None,
            };

            if let Some(next_spawns) = next_spawns {
                // Continue spawning wave
                self.model.wave_delay = wave.between_delay;
                for unit_type in [next_spawns] {
                    let unit =
                        self.spawn_unit(&unit_type, Faction::Enemy, Position::zero(Faction::Enemy));
                    let unit = self.model.units.get_mut(&unit).unwrap();
                    let round = &self.model.round;
                    let statuses = round
                        .statuses
                        .iter()
                        .chain(round.waves.front().unwrap().statuses.iter())
                        .map(|status| {
                            status.get(&self.model.statuses).clone().attach(
                                Some(unit.id),
                                None,
                                &mut self.model.next_id,
                            )
                        });
                    unit.all_statuses.extend(statuses);
                }
            } else {
                // Next wave
                round.waves.pop_front();
                if let Some(next_wave) = round.waves.front() {
                    self.model.wave_delay = next_wave.start_delay;
                }
            }
        } else if !self
            .model
            .units
            .iter()
            .any(|unit| unit.faction != Faction::Player)
            && self.model.time_bombs.is_empty()
            && self.effects.is_empty()
        {
            // Next round
            self.model.transition = true;
        }
    }
}
