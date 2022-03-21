use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct HealEffect {
    pub hp: DamageValue,
}

impl EffectContainer for HealEffect {
    fn walk_effects_mut(&mut self, _f: &mut dyn FnMut(&mut Effect)) {}
}

impl EffectImpl for HealEffect {
    fn process(self: Box<Self>, context: EffectContext, logic: &mut logic::Logic) {
        let effect = *self;
        let target_unit = context
            .target
            .and_then(|id| logic.model.units.get_mut(&id))
            .expect("Target not found");

        let heal = target_unit.max_hp * effect.hp.relative + effect.hp.absolute;
        let heal = min(heal, target_unit.max_hp - target_unit.hp);
        target_unit.hp += heal;
    }
}
