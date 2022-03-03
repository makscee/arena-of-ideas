use super::*;

impl Game {
    pub fn process_deaths(&mut self) {
        for id in self.units.ids().copied().collect::<Vec<Id>>() {
            let mut unit = self.units.remove(&id).unwrap();
            if unit.hp <= 0 {
                let effects = unit.death_effects.clone();
                for effect in effects {
                    self.apply_effect(&effect, None, &mut unit);
                }
            }
            self.units.insert(unit);
        }
        self.units.retain(|unit| unit.hp > 0);
    }
}
