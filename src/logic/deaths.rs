use super::*;

impl Logic<'_> {
    pub fn kill(&mut self, id: Id) {
        let unit = self.model.units.get_mut(&id).unwrap();
        unit.health = Health::new(0.0);
        let unit = self.model.units.get(&id).unwrap();

        for (effect, vars, status_id) in unit
            .all_statuses
            .iter()
            .flat_map(|status| (status.trigger(|trigger| matches!(trigger, StatusTrigger::Death))))
        {
            self.effects.push_front(QueuedEffect {
                effect,
                context: EffectContext {
                    caster: Some(unit.id),
                    from: Some(unit.id),
                    target: Some(unit.id),
                    vars,
                    status_id: Some(status_id),
                },
            });
        }

        for other in self.model.units.iter().filter(|other| other.id != unit.id) {
            for (effect, vars, status_id) in other.all_statuses.iter().flat_map(|status| {
                status.trigger(|trigger| match trigger {
                    StatusTrigger::Scavenge { who, range, clan } => {
                        who.matches(other.faction, unit.faction)
                            && clan.map(|clan| unit.clans.contains(&clan)).unwrap_or(true)
                            && distance_between_units(other, unit) > *range
                    }
                    _ => false,
                })
            }) {
                self.effects.push_front(QueuedEffect {
                    effect,
                    context: EffectContext {
                        caster: Some(other.id),
                        from: Some(other.id),
                        target: Some(unit.id),
                        vars,
                        status_id: Some(status_id),
                    },
                })
            }
        }
    }
    pub fn process_deaths(&mut self) {
        for unit in &self.model.units {
            if unit.health <= Health::ZERO {
                self.model.dead_units.insert(unit.clone());
            }
        }
        self.model.units.retain(|unit| unit.health > Health::ZERO);
    }
}
