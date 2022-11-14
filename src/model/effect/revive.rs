use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ReviveEffect {
    pub health: Expr,
}

impl EffectContainer for ReviveEffect {
    fn walk_effects_mut(&mut self, _f: &mut dyn FnMut(&mut Effect)) {}
}

impl EffectImpl for ReviveEffect {
    fn process(self: Box<Self>, context: EffectContext, logic: &mut logic::Logic) {
        let effect = *self;
        let health = effect.health.calculate(&context, &logic.model);
        let unit = logic.model.dead_units.remove(&context.target);
        if unit.is_none() {
            warn!("Tried to revive unit#{} that is not dead", context.target);
        }
        let mut unit = unit.unwrap();
        unit.stats.health = health;
        unit.permanent_stats.health = health;
        assert!(
            !logic.model.units.iter().any(|u| u.id == unit.id),
            "Tried to revive unit#{} that is alive",
            unit.id
        );
        logic.model.units.insert(unit); // TODO: check validity
    }
}
