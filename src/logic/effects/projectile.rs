pub use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ProjectileEffect {
    pub speed: Coord,
    pub effect: Effect,
}

impl ProjectileEffect {
    pub fn walk_children_mut(&mut self, f: &mut impl FnMut(&mut Effect)) {
        self.effect.walk_mut(f);
    }
}

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
        self.model.projectiles.insert(Projectile {
            id: self.model.next_id,
            attacker: from.id,
            target: target.id,
            position: from.position + (target.position - from.position).normalize() * from.radius(),
            speed: effect.speed,
            target_position: target.position,
            effect: effect.effect.clone(),
        });
        self.model.next_id += 1;
    }
}
