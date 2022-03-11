use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct WeakenModifier {
    pub percent: R32,
}

impl WeakenModifier {
    pub fn apply(&self, effect: &mut Effect) {
        let multiplier = R32::ONE - self.percent / r32(100.0);
        match effect {
            Effect::Noop => {}
            Effect::Damage(effect) => match &mut effect.hp {
                DamageValue::Absolute(value) => *value *= multiplier,
                DamageValue::Relative(value) => *value *= multiplier,
            },
            Effect::Projectile(effect) => {
                self.apply(&mut effect.effect);
            }
            Effect::AddStatus(_) => {}
            Effect::Spawn(_) => {}
            Effect::AOE(effect) => {
                self.apply(&mut effect.effect);
            }
            Effect::TimeBomb(effect) => {
                self.apply(&mut effect.effect);
            }
            Effect::Suicide(_) => {}
            Effect::Chain(effect) => {
                self.apply(&mut effect.effect);
            }
            Effect::Repeat { effect, .. } => {
                self.apply(effect);
            }
            Effect::List { effects } => {
                for effect in effects {
                    self.apply(effect);
                }
            }
            Effect::Random { choices } => {
                for choice in choices {
                    self.apply(&mut choice.effect);
                }
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum Modifier {
    Weaken(WeakenModifier),
}

impl Effect {
    pub fn apply_modifier(&mut self, modifier: &Modifier) {
        match modifier {
            Modifier::Weaken(modifier) => modifier.apply(self),
        }
    }
}
