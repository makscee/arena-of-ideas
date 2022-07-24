use super::*;

impl Logic {
    pub fn process_round(&mut self) {
        if let Some(round) = self.model.round.clone() {
            self.model.round = None;
            for unit_type in &round.enemies {
                let unit =
                    self.spawn_unit(&unit_type, Faction::Enemy, Position::zero(Faction::Enemy));
                let unit = self.model.units.get_mut(&unit).unwrap();
                let statuses = round.statuses.iter().map(|status| {
                    status.get(&self.model.statuses).clone().attach(
                        Some(unit.id),
                        None,
                        &mut self.model.next_id,
                    )
                });
                unit.all_statuses.extend(statuses);
            }
        } else {
            if !self
                .model
                .units
                .iter()
                .any(|unit| unit.faction != Faction::Player)
                && self.effects.is_empty()
            {
                // Next round
                self.model.transition = true;
            }
        }
    }
}
