use super::*;

impl Game {
    pub fn process_projectiles(&mut self) {
        let mut delete_projectiles = Vec::new();
        for id in self.projectiles.ids().copied().collect::<Vec<Id>>() {
            let mut projectile = self.projectiles.remove(&id).unwrap();

            let mut attacker = self.units.remove(&projectile.attacker);
            if let Some(mut target) = self.units.remove(&projectile.target) {
                projectile.target_position = target.position;
                if (projectile.position - target.position).len() < target.radius() {
                    self.deal_damage(
                        attacker.as_mut(),
                        &mut target,
                        &projectile.effects,
                        &projectile.kill_effects,
                        projectile.damage,
                    );
                    delete_projectiles.push(projectile.id);
                }
                self.units.insert(target);
            }
            if let Some(attacker) = attacker {
                self.units.insert(attacker);
            }
            let max_distance = projectile.speed * self.delta_time;
            let distance = (projectile.target_position - projectile.position).len();
            if distance < max_distance {
                delete_projectiles.push(projectile.id);
            }
            projectile.position += (projectile.target_position - projectile.position)
                .clamp_len(..=projectile.speed * self.delta_time);

            self.projectiles.insert(projectile);
        }
        for id in delete_projectiles {
            self.projectiles.remove(&id);
        }
    }
}
