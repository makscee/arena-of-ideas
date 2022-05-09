use super::*;

impl Logic<'_> {
    pub fn process_actions(&mut self) {
        self.process_units(Self::process_unit_actions);
    }
    fn process_unit_actions(&mut self, unit: &mut Unit) {
        if let ActionState::Start { time, target } = &mut unit.action_state {
            *time += self.delta_time;
            if *time > unit.action.animation_delay {
                if let Some(target) = self.model.units.get(target) {
                    let mut effect = unit.action.effect.clone();
                    for modifier in mem::take(&mut unit.next_action_modifiers) {
                        effect.apply_modifier(&modifier);
                    }
                    for status in &unit.all_statuses {
                        if let Status::Modifier(status) = status {
                            effect.apply_modifier(&status.modifier);
                        }
                    }
                    self.effects.push_back(QueuedEffect {
                        effect,
                        context: EffectContext {
                            caster: Some(unit.id),
                            from: Some(unit.id),
                            target: Some(target.id),
                            vars: default(),
                        },
                    });
                }
                unit.last_action_time = self.model.time;
                unit.action_state = ActionState::Cooldown {
                    time: Time::new(0.0),
                };
            } else {
                if let Some(target) = self.model.dead_units.get(target) {
                    unit.action_state = ActionState::None;
                }
            }
        }
    }

    pub fn process_cooldowns(&mut self) {
        self.process_units(Self::process_unit_cooldowns);
    }
    fn process_unit_cooldowns(&mut self, unit: &mut Unit) {
        if let ActionState::Cooldown { time } = &mut unit.action_state {
            let attack_speed = unit.all_statuses.iter().fold(1.0, |speed, status| {
                speed
                    + if let Status::AttackSpeed(status) = status {
                        status.percent / 100.0
                    } else {
                        0.0
                    }
            });
            *time += self.delta_time * r32(attack_speed);
            if *time > unit.action.cooldown {
                unit.action_state = ActionState::None;
            }
        }
    }
}
