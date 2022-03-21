use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ChainEffect {
    pub targets: usize,
    pub jump_distance: Coord,
    pub effect: Effect,
    pub jump_modifier: Modifier,
}

impl EffectContainer for ChainEffect {
    fn walk_effects_mut(&mut self, f: &mut dyn FnMut(&mut Effect)) {
        self.effect.walk_mut(f);
    }
}

impl EffectImpl for ChainEffect {
    fn process(self: Box<Self>, context: EffectContext, logic: &mut logic::Logic) {
        let effect = *self;
        let mut touched = HashSet::new();
        let mut touch_effect = effect.effect.clone();
        let mut target = logic.model.units.get(&context.target.unwrap()).unwrap();
        let mut effects = Vec::new();
        while touched.len() < effect.targets {
            touched.insert(target.id);
            effects.push(QueuedEffect {
                effect: touch_effect.clone(),
                context: EffectContext {
                    target: Some(target.id),
                    ..context
                },
            });
            touch_effect.apply_modifier(&effect.jump_modifier);
            target = match logic
                .model
                .units
                .iter()
                .filter(|unit| unit.faction == target.faction && !touched.contains(&unit.id))
                .filter(|unit| (unit.position - target.position).len() < effect.jump_distance)
                .min_by_key(|unit| (unit.position - target.position).len())
            {
                Some(unit) => unit,
                None => break,
            }
        }
        for effect in effects.into_iter().rev() {
            logic.effects.push_front(effect);
        }
    }
}
