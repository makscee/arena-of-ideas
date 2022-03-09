use super::*;

impl Logic<'_> {
    pub fn process_attacks(&mut self) {
        self.process_units(Self::process_unit_attacks);
    }
    fn process_unit_attacks(&mut self, unit: &mut Unit) {
        if let AttackState::Start { time, target } = &mut unit.attack_state {
            *time += self.delta_time;
            if *time > unit.attack.animation_delay {
                let target = self.model.units.remove(target);
                unit.attack_state = AttackState::Cooldown {
                    time: Time::new(0.0),
                };
                if let Some(mut target) = target {
                    if let Some(projectile_speed) = unit.projectile_speed {
                        self.model.projectiles.insert(Projectile {
                            id: self.model.next_id,
                            attacker: unit.id,
                            target: target.id,
                            position: unit.position
                                + (target.position - unit.position).normalize() * unit.radius(),
                            speed: projectile_speed,
                            target_position: target.position,
                            effects: unit.attack.effects.clone(),
                        });
                        self.model.next_id += 1;
                    } else {
                        for effect in &unit.attack.effects {
                            self.effects.push(QueuedEffect {
                                effect: effect.clone(),
                                caster: Some(unit.id),
                                target: Some(target.id),
                            });
                        }
                    }
                    self.model.units.insert(target);
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
            if *time > unit.attack.cooldown {
                unit.attack_state = AttackState::None;
            }
        }
    }
}
