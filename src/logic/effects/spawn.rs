use super::*;

impl Logic<'_> {
    pub fn process_spawn_effect(
        &mut self,
        QueuedEffect { effect, context }: QueuedEffect<SpawnEffect>,
    ) {
        let caster = context
            .caster
            .and_then(|id| self.model.units.get(&id).or(self.model.dead_units.get(&id)))
            .expect("Caster not found");
        let faction = caster.faction;
        let target = context
            .target
            .and_then(|id| self.model.units.get(&id).or(self.model.dead_units.get(&id)))
            .expect("Target not found");
        let position = target.position;
        self.spawn_unit(&effect.unit_type, faction, position);
    }
}
