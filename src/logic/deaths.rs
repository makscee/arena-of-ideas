use super::*;

impl Game {
    pub fn process_deaths(&mut self) {
        self.process_units(Self::process_unit_death);
        self.units.retain(|unit| unit.hp > Health::new(0.0));
    }
    fn process_unit_death(&mut self, unit: &mut Unit) {
        if unit.hp <= Health::new(0.0) {
            let effects = unit.death_effects.clone();
            for effect in effects {
                self.apply_effect(&effect, None, unit);
            }
        }
    }
}
