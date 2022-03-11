use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct WeakenModifier {
    pub percent: R32,
}

impl WeakenModifier {
    pub fn apply(&self, effect: &mut Effect) {
        let multiplier = R32::ONE - self.percent / r32(100.0);
        effect.walk_mut(&mut |effect| match effect {
            Effect::Damage(damage) => damage.hp = damage.hp * multiplier,
            _ => {}
        });
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
