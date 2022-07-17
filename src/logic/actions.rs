use super::*;

impl Logic<'_> {
    pub fn process_actions(&mut self) {
        self.process_units(Self::process_unit_actions);
    }
    fn process_unit_actions(&mut self, unit: &mut Unit) {
        if let ActionState::Start { target } = &mut unit.action_state {
            if unit
                .flags
                .iter()
                .any(|flag| matches!(flag, UnitStatFlag::ActionUnable))
            {
                return;
            }
            if let Some(target) = self.model.units.get(target) {
                let mut effect = unit.action.effect.clone();
                for modifier in mem::take(&mut unit.next_action_modifiers) {
                    effect.apply_modifier(&modifier);
                }
                for (effect, vars, status_id) in unit.all_statuses.iter().flat_map(|status| {
                    status.trigger(|trigger| matches!(trigger, StatusTrigger::Action))
                }) {
                    self.effects.push_front(QueuedEffect {
                        effect,
                        context: EffectContext {
                            caster: Some(unit.id),
                            from: Some(unit.id),
                            target: Some(target.id),
                            vars,
                            status_id: Some(status_id),
                        },
                    });
                }
                self.effects.push_back(QueuedEffect {
                    effect,
                    context: EffectContext {
                        caster: Some(unit.id),
                        from: Some(unit.id),
                        target: Some(target.id),
                        vars: default(),
                        status_id: None,
                    },
                });
            }
            unit.last_action_time = self.model.time;
            unit.action_state = ActionState::Cooldown { time: 0 };
        }
    }

    pub fn tick_cooldowns(&mut self) {
        self.process_units(Self::tick_unit_cooldowns);
        let mut units: Vec<Id> = self.model.units.ids().copied().collect();
        units.shuffle(&mut global_rng());
        self.model.current_tick.action_queue = units.into_iter().collect();
    }
    fn tick_unit_cooldowns(&mut self, unit: &mut Unit) {
        if let ActionState::Cooldown { time } = &mut unit.action_state {
            *time += 1;
            if *time > unit.action.cooldown {
                unit.action_state = ActionState::None;
            }
        }
    }
}
