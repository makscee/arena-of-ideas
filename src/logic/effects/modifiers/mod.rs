use super::*;

impl StrengthModifier {
    pub fn apply(&self, effect: &mut Effect) {
        effect.walk_mut(&mut |effect| match effect {
            Effect::Damage(damage) => damage.hp = damage.hp * self.multiplier + self.add,
            _ => {}
        });
    }
}

impl Effect {
    pub fn apply_modifier(&mut self, modifier: &Modifier) {
        match modifier {
            Modifier::Strength(modifier) => modifier.apply(self),
        }
    }
}
