use super::*;

impl StrengthModifier {
    pub fn apply(&self, effect: &mut Effect) {
        effect.walk_mut(&mut |effect| match effect {
            Effect::Damage(damage) => {
                damage.value = Expr::WithVar {
                    name: VarName::Value,
                    value: Box::new(damage.value.clone()),
                    result: Box::new(self.value.clone()),
                }
            }
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
