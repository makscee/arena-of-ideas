use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ProjectileEffect {
    pub speed: Coord,
    pub effect: Effect,
}

impl EffectContainer for ProjectileEffect {
    fn walk_effects_mut(&mut self, f: &mut dyn FnMut(&mut Effect)) {
        self.effect.walk_mut(f);
    }
}

impl EffectImpl for ProjectileEffect {
    fn process(self: Box<Self>, context: EffectContext, logic: &mut logic::Logic) {
        let effect = *self;
        let target = context
            .target
            .and_then(|id| logic.model.units.get(&id))
            .expect("Target not found");
        let from = context
            .from
            .and_then(|id| logic.model.units.get(&id))
            .expect("Caster not found");
        assert_ne!(target.id, from.id);
        logic.model.projectiles.insert(Projectile {
            id: logic.model.next_id,
            caster: from.id,
            target: target.id,
            position: from.position + (target.position - from.position).normalize() * from.radius(),
            speed: effect.speed,
            target_position: target.position,
            effect: effect.effect.clone(),
        });
        logic.model.next_id += 1;
    }
}
