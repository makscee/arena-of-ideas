use super::*;

impl Logic<'_> {
    pub fn process_deaths(&mut self) {
        self.process_units(Self::process_unit_death);
        for unit in &self.model.units {
            if unit.hp <= Health::new(0.0) {
                self.model.dead_units.insert(unit.clone());
            }
        }
        self.model.units.retain(|unit| unit.hp > Health::new(0.0));
    }
    fn process_unit_death(&mut self, unit: &mut Unit) {
        if unit.hp <= Health::new(0.0) {
            for effect in &unit.death_effects {
                self.effects.push(QueuedEffect {
                    effect: effect.clone(),
                    caster: Some(unit.id),
                    target: Some(unit.id),
                });
            }
        }
    }
}
