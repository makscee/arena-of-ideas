use super::*;

impl Logic<'_> {
    pub fn kill(&mut self, id: Id) {
        let unit = self.model.units.get_mut(&id).unwrap();
        unit.hp = Health::new(0.0);
        for status in &unit.all_statuses {
            if let Status::DeathRattle(effect) = status {
                self.effects.push_front(QueuedEffect {
                    effect: effect.clone(),
                    context: EffectContext {
                        caster: Some(unit.id),
                        from: Some(unit.id),
                        target: Some(unit.id),
                    },
                });
            }
        }
    }
    pub fn process_deaths(&mut self) {
        for unit in &self.model.units {
            if unit.hp <= Health::ZERO {
                self.model.dead_units.insert(unit.clone());
            }
        }
        self.model.units.retain(|unit| unit.hp > Health::ZERO);
    }
}
