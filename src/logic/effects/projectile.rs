pub use super::*;

impl Logic<'_> {
    pub fn process_projectile_effect(
        &mut self,
        QueuedEffect { effect, context }: QueuedEffect<ProjectileEffect>,
    ) {
        let target = context
            .target
            .and_then(|id| self.model.units.get(&id))
            .expect("Target not found");
        let from = context
            .from
            .and_then(|id| self.model.units.get(&id))
            .expect("Caster not found");
        assert_ne!(target.id, from.id);
        self.model.projectiles.insert(Projectile {
            id: self.model.next_id,
            caster: from.id,
            target: target.id,
            position: from.position + (target.position - from.position).normalize() * from.radius(),
            speed: effect.speed,
            target_position: target.position,
            effect: effect.effect.clone(),
        });
        self.model.next_id += 1;
    }
}
