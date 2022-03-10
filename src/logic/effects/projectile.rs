pub use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ProjectileEffect {
    pub speed: Coord,
    pub effect: Effect,
}

impl Logic<'_> {
    pub fn process_projectile_effect(
        &mut self,
        QueuedEffect {
            effect,
            caster,
            target,
        }: QueuedEffect<ProjectileEffect>,
    ) {
        let target = target
            .and_then(|id| self.model.units.get(&id))
            .expect("Target not found");
        let caster = caster
            .and_then(|id| self.model.units.get(&id))
            .expect("Caster not found");
        self.model.projectiles.insert(Projectile {
            id: self.model.next_id,
            attacker: caster.id,
            target: target.id,
            position: caster.position
                + (target.position - caster.position).normalize() * caster.radius(),
            speed: effect.speed,
            target_position: target.position,
            effect: effect.effect.clone(),
        });
        self.model.next_id += 1;
    }
}
