use super::*;

impl Logic<'_> {
    pub fn process_attacks(&mut self) {
        self.process_units(Self::process_unit_attacks);
    }
    fn process_unit_attacks(&mut self, unit: &mut Unit) {
        if let AttackState::Start { time, target } = &mut unit.attack_state {
            *time += self.delta_time;
            if *time > unit.attack.animation_delay {
                if let Some(target) = self.model.units.get(target) {
                    let mut effect = unit.attack.effect.clone();
                    for status in &unit.all_statuses {
                        if let Status::Modifier(modifier) = status {
                            effect.apply_modifier(modifier);
                        }
                    }
                    self.effects.push(QueuedEffect {
                        effect,
                        caster: Some(unit.id),
                        target: Some(target.id),
                    });
                }
                unit.attack_state = AttackState::Cooldown {
                    time: Time::new(0.0),
                };
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
