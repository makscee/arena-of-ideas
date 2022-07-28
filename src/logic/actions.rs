use super::*;

impl Logic {
    pub fn process_actions(&mut self) {
        self.process_units_random(Self::process_unit_actions);
    }
    fn process_unit_actions(&mut self, unit: &mut Unit) {
        if self.model.current_tick.visual_timer > Time::new(0.0) {
            return;
        }
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
            match unit.faction {
                Faction::Player => self.model.last_player_action_time = self.model.time,
                Faction::Enemy => self.model.last_enemy_action_time = self.model.time,
            }
            unit.action_state = ActionState::Cooldown { time: 0 };
            self.model.current_tick.visual_timer += Time::new(UNIT_VISUAL_TIME);
        }
    }
}
