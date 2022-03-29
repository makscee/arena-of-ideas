use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct SplashEffect {
    pub degrees: R32,
    pub effect: Effect,
    pub effect_on_caster: Effect,
}

impl EffectContainer for SplashEffect {
    fn walk_effects_mut(&mut self, f: &mut dyn FnMut(&mut Effect)) {
        self.effect.walk_mut(f);
    }
}

impl EffectImpl for SplashEffect {
    fn process(self: Box<Self>, context: EffectContext, logic: &mut logic::Logic) {
        let effect = *self;
        let from = context
            .from
            .and_then(|id| logic.model.units.get(&id))
            .expect("From not found");
        let target = context
            .target
            .and_then(|id| logic.model.units.get(&id))
            .expect("Target not found");
        let mut target_count = 0;
        for unit in &logic.model.units {
            if unit.id == from.id {
                continue;
            }
            if unit.faction != target.faction {
                continue;
            }
            if distance_between_units(unit, from) > from.action.range {
                continue;
            }
            // TODO: this only checks that center is in angle
            if Vec2::dot(
                (unit.position - from.position).normalize(),
                (target.position - from.position).normalize(),
            )
            .raw()
            .acos()
                > effect.degrees.raw() * f32::PI / 180.0
            {
                continue;
            }
            logic.effects.push_front(QueuedEffect {
                effect: effect.effect.clone(),
                context: EffectContext {
                    target: Some(unit.id),
                    ..context.clone()
                },
            });
            target_count += 1;
        }
        logic.effects.push_front(QueuedEffect {
            effect: effect.effect_on_caster,
            context: {
                let mut context = context.clone();
                context.target = context.caster;
                context
                    .vars
                    .insert(VarName::TargetCount, r32(target_count as f32));
                context
            },
        })
    }
}
