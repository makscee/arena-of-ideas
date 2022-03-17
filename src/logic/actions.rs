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
                    for status in &unit.all_statuses {
                        if let Status::Modifier(modifier) = status {
                            effect.apply_modifier(modifier);
                        }
                    }
                    self.effects.push_back(QueuedEffect {
                        effect,
                        context: EffectContext {
                            caster: Some(unit.id),
                            from: Some(unit.id),
                            target: Some(target.id),
                        },
                    });
                }
                unit.action_state = ActionState::Cooldown {
                    time: Time::new(0.0),
                };
            }
        }
    }

    pub fn process_cooldowns(&mut self) {
        self.process_units(Self::process_unit_cooldowns);
    }
    fn process_unit_cooldowns(&mut self, unit: &mut Unit) {
        if let ActionState::Cooldown { time } = &mut unit.action_state {
            *time += self.delta_time;
            if *time > unit.action.cooldown {
                unit.action_state = ActionState::None;
            }
        }
    }
}
