use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SpawnEffect {
    pub unit_type: UnitType,
}

impl Logic<'_> {
    pub fn process_spawn_effect(
        &mut self,
        QueuedEffect {
            target,
            caster,
            effect,
        }: QueuedEffect<SpawnEffect>,
    ) {
        let caster = caster
            .and_then(|id| self.model.units.get(&id).or(self.model.dead_units.get(&id)))
            .expect("Caster not found");
        let faction = caster.faction;
        let target = target
            .and_then(|id| self.model.units.get(&id).or(self.model.dead_units.get(&id)))
            .expect("Target not found");
        let position = target.position;
        self.spawn_unit(&effect.unit_type, faction, position);
    }
}
