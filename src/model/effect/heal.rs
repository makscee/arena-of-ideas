use super::*;

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct HealEffect {
    #[serde(default)]
    pub hp: DamageValue,
    #[serde(default)]
    pub heal_past_max: DamageValue,
    #[serde(default)]
    pub max_hp: DamageValue,
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

        let max_heal = target_unit.max_hp * effect.max_hp.relative + effect.max_hp.absolute;
        target_unit.max_hp += max_heal;

        let heal = target_unit.max_hp * effect.hp.relative + effect.hp.absolute;
        let max_heal = target_unit.max_hp
            + target_unit.max_hp * effect.heal_past_max.relative
            + effect.heal_past_max.absolute;
        let heal = min(heal, max_heal - target_unit.hp);
        target_unit.hp += heal;

        for status in &target_unit.all_statuses {
            if let Status::OnHeal(status) = status {
                logic.effects.push_front(QueuedEffect {
                    effect: status.effect.clone(),
                    context: context.clone(),
                });
            }
        }
    }
}
