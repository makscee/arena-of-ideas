use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TimeBombEffect {
    pub time: Time,
    pub effects: Vec<Effect>,
}

impl Game {
    pub fn process_time_bomb_effect(
        &mut self,
        QueuedEffect {
            target,
            caster,
            effect,
        }: QueuedEffect<TimeBombEffect>,
    ) {
        let target = target
            .and_then(|id| self.units.get(&id).or(self.dead_units.get(&id)))
            .expect("Target not found");
        self.time_bombs.insert(TimeBomb {
            id: self.next_id,
            position: target.position,
            caster,
            time: effect.time,
            effects: effect.effects,
        });
        self.next_id += 1;
    }
}
