use super::*;

impl Logic<'_> {
    pub fn process_time_bombs(&mut self) {
        for bomb in &mut self.model.time_bombs {
            bomb.time -= self.delta_time;
            if bomb.time <= Time::ZERO {
                self.effects.push_back(QueuedEffect {
                    effect: bomb.effect.clone(),
                    context: EffectContext {
                        caster: bomb.caster,
                        from: Some(bomb.id),
                        target: Some(bomb.id),
                        vars: default(),
                    },
                });
                self.model.dead_time_bombs.insert(bomb.clone());
            }
        }
        self.model.time_bombs.retain(|bomb| bomb.time > Time::ZERO);
    }
}
