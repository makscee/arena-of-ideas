use super::*;

impl Game {
    pub fn process_projectiles(&mut self) {
        let mut delete_projectiles = Vec::new();
        for id in self.model.projectiles.ids().copied().collect::<Vec<Id>>() {
            let mut projectile = self.model.projectiles.remove(&id).unwrap();

            let mut attacker = self.model.units.remove(&projectile.attacker);
            if let Some(mut target) = self.model.units.remove(&projectile.target) {
                projectile.target_position = target.position;
                if (projectile.position - target.position).len() < target.radius() {
                    for effect in &projectile.effects {
                        self.effects.push(QueuedEffect {
                            effect: effect.clone(),
                            caster: Some(projectile.attacker),
                            target: Some(target.id),
                        });
                    }
                    delete_projectiles.push(projectile.id);
                }
                self.model.units.insert(target);
            }
            if let Some(attacker) = attacker {
                self.model.units.insert(attacker);
            }
            let max_distance = projectile.speed * self.delta_time;
            let distance = (projectile.target_position - projectile.position).len();
            if distance < max_distance {
                delete_projectiles.push(projectile.id);
            }
            projectile.position += (projectile.target_position - projectile.position)
                .clamp_len(..=projectile.speed * self.delta_time);

            self.model.projectiles.insert(projectile);
        }
        for id in delete_projectiles {
            self.model.projectiles.remove(&id);
        }
    }
}
