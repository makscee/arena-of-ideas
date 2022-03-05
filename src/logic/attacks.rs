use super::*;

impl Game {
    pub fn process_attacks(&mut self) {
        self.process_units(Self::process_unit_attacks);
    }
    fn process_unit_attacks(&mut self, unit: &mut Unit) {
        if let AttackState::Start { time, target } = &mut unit.attack_state {
            *time += self.delta_time;
            if *time > unit.attack_animation_delay {
                let target = self.units.remove(target);
                unit.attack_state = AttackState::Cooldown {
                    time: Time::new(0.0),
                };
                if let Some(mut target) = target {
                    if let Some(projectile_speed) = unit.projectile_speed {
                        self.projectiles.insert(Projectile {
                            id: self.next_id,
                            attacker: unit.id,
                            target: target.id,
                            position: unit.position
                                + (target.position - unit.position).normalize() * unit.radius(),
                            speed: projectile_speed,
                            target_position: target.position,
                            effects: unit.attack_effects.clone(),
                        });
                        self.next_id += 1;
                    } else {
                        let effects = unit.attack_effects.clone();
                        self.deal_damage(Some(unit), &mut target, &effects);
                    }
                    self.units.insert(target);
                }
            }
        }
    }

    pub fn process_cooldowns(&mut self) {
        self.process_units(Self::process_unit_cooldowns);
    }
    fn process_unit_cooldowns(&mut self, unit: &mut Unit) {
        if let AttackState::Cooldown { time } = &mut unit.attack_state {
            *time += self.delta_time;
            if *time > unit.attack_cooldown {
                unit.attack_state = AttackState::None;
            }
        }
    }
}
