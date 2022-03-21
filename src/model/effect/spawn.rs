use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SpawnEffect {
    pub unit_type: UnitType,
}

impl EffectContainer for SpawnEffect {
    fn walk_effects_mut(&mut self, _f: &mut dyn FnMut(&mut Effect)) {}
}

impl EffectImpl for SpawnEffect {
    fn process(self: Box<Self>, context: EffectContext, logic: &mut logic::Logic) {
        let effect = *self;
        let caster = context
            .caster
            .and_then(|id| {
                logic
                    .model
                    .units
                    .get(&id)
                    .or(logic.model.dead_units.get(&id))
            })
            .expect("Caster not found");
        let faction = caster.faction;
        let target = context
            .target
            .and_then(|id| {
                logic
                    .model
                    .units
                    .get(&id)
                    .or(logic.model.dead_units.get(&id))
            })
            .expect("Target not found");
        let position = target.position;
        logic.spawn_unit(&effect.unit_type, faction, position);
    }
}
