use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ChainEffect {
    pub targets: usize,
    pub split_probability: R32,
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
        let target = logic.model.units.get(&context.target.unwrap()).unwrap();
        let mut effects = Vec::new();
        let mut queue = std::collections::VecDeque::new();
        queue.push_back((0, target));
        touched.insert(target.id);
        while let Some((chain_len, target)) = queue.pop_front() {
            effects.push(QueuedEffect {
                effect: {
                    let mut current_effect = effect.effect.clone();
                    for _ in 0..chain_len {
                        current_effect.apply_modifier(&effect.jump_modifier);
                    }
                    current_effect
                },
                context: EffectContext {
                    target: Some(target.id),
                    ..context.clone()
                },
            });
            if chain_len < effect.targets {
                for probability in [1.0, effect.split_probability.as_f32()] {
                    if global_rng().gen_bool(probability as f64) {
                        if let Some(next) = logic
                            .model
                            .units
                            .iter()
                            .filter(|unit| {
                                unit.faction == target.faction && !touched.contains(&unit.id)
                            })
                            .filter(|unit| {
                                distance_between_units(unit, target) < effect.jump_distance
                            })
                            .min_by_key(|unit| distance_between_units(unit, target))
                        {
                            queue.push_back((chain_len + 1, next));
                            touched.insert(next.id);
                        }
                    }
                }
            }
        }
        for effect in effects.into_iter().rev() {
            logic.effects.push_front(effect);
        }
    }
}
