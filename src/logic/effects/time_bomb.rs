use super::*;

impl Logic<'_> {
    pub fn process_time_bomb_effect(
        &mut self,
        QueuedEffect { effect, context }: QueuedEffect<TimeBombEffect>,
    ) {
        let target = context
            .target
            .and_then(|id| self.model.units.get(&id).or(self.model.dead_units.get(&id)))
            .expect("Target not found");
        self.model.time_bombs.insert(TimeBomb {
            id: self.model.next_id,
            position: target.position,
            caster: context.caster,
            time: effect.time,
            effect: effect.effect,
        });
        self.model.next_id += 1;
    }
}
