use super::*;

impl Logic<'_> {
    pub fn kill(&mut self, id: Id) {
        let unit = self.model.units.get_mut(&id).unwrap();
        unit.health = Health::new(0.0);
        let unit = self.model.units.get(&id).unwrap();
        for status in &unit.all_statuses {
            if let Status::OnDeath(status) = status {
                self.effects.push_front(QueuedEffect {
                    effect: status.effect.clone(),
                    context: EffectContext {
                        caster: Some(unit.id),
                        from: Some(unit.id),
                        target: Some(unit.id),
                        vars: default(),
                    },
                });
            }
        }
        for other in &self.model.units {
            if other.id == unit.id {
                continue;
            }
            for status in &other.all_statuses {
                if let Status::Scavenge(status) = status {
                    if !status.who.matches(other.faction, unit.faction) {
                        continue;
                    }
                    if let Some(clan) = status.clan {
                        if !unit.clans.contains(&clan) {
                            continue;
                        }
                    }
                    if distance_between_units(other, unit) > status.range {
                        continue;
                    }
                    self.effects.push_back(QueuedEffect {
                        effect: status.effect.clone(),
                        context: EffectContext {
                            caster: Some(other.id),
                            from: Some(other.id),
                            target: Some(unit.id),
                            vars: default(),
                        },
                    })
                }
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
