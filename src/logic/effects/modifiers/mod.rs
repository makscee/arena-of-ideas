use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct WeakenEffectModifier {
    pub percent: R32,
}

impl WeakenEffectModifier {
    pub fn apply(&self, effect: &mut Effect) {
        let multiplier = R32::ONE - self.percent / r32(100.0);
        match effect {
            Effect::Damage(effect) => match &mut effect.hp {
                DamageValue::Absolute(value) => *value *= multiplier,
                DamageValue::Relative(value) => *value *= multiplier,
            },
            Effect::AddStatus(_) => {}
            Effect::Spawn(_) => {}
            Effect::AOE(effect) => {
                for effect in &mut effect.effects {
                    self.apply(effect);
                }
            }
            Effect::TimeBomb(effect) => {
                for effect in &mut effect.effects {
                    self.apply(effect);
                }
            }
            Effect::Suicide(_) => {}
            Effect::Chain(effect) => {
                for effect in &mut effect.effects {
                    self.apply(effect);
                }
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum EffectModifier {
    Weaken(WeakenEffectModifier),
}

impl Effect {
    pub fn apply_modifier(&mut self, modifier: &EffectModifier) {
        match modifier {
            EffectModifier::Weaken(modifier) => modifier.apply(self),
        }
    }
}
