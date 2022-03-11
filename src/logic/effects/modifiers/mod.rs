use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct StrengthModifier {
    pub multiplier: R32,
    pub add: R32,
}

impl StrengthModifier {
    pub fn apply(&self, effect: &mut Effect) {
        effect.walk_mut(&mut |effect| match effect {
            Effect::Damage(damage) => damage.hp = damage.hp * self.multiplier + self.add,
            _ => {}
        });
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum Modifier {
    Strength(StrengthModifier),
}

impl Effect {
    pub fn apply_modifier(&mut self, modifier: &Modifier) {
        match modifier {
            Modifier::Strength(modifier) => modifier.apply(self),
        }
    }
}
