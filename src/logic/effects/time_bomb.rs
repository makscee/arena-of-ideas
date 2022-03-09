use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TimeBombEffect {
    pub time: Time,
    pub effect: Effect,
}

impl Logic<'_> {
    pub fn process_time_bomb_effect(
        &mut self,
        QueuedEffect {
            target,
            caster,
            effect,
        }: QueuedEffect<TimeBombEffect>,
    ) {
        let target = target
            .and_then(|id| self.model.units.get(&id).or(self.model.dead_units.get(&id)))
            .expect("Target not found");
        self.model.time_bombs.insert(TimeBomb {
            id: self.model.next_id,
            position: target.position,
            caster,
            time: effect.time,
            effect: effect.effect,
        });
        self.model.next_id += 1;
    }
}
