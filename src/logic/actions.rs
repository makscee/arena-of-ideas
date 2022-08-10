use super::*;

impl Logic {
    pub fn process_actions(&mut self) {
        self.process_units_sorted(Self::process_unit_actions);
    }
    fn process_unit_actions(&mut self, unit: &mut Unit) {
        if self.model.current_tick.visual_timer > Time::new(0.0) {
            return;
        }
        if let Some(actor) = self.model.acting_unit {
            if actor != unit.id {
                return;
            } else {
                self.model.acting_unit = None;
                return;
            }
        }
        match &mut unit.action_state {
            ActionState::Start { target } => {
                if unit
                    .flags
                    .iter()
                    .any(|flag| matches!(flag, UnitStatFlag::ActionUnable))
                {
                    return;
                }
                if let Some(target) = self.model.units.get(target) {
                    let mut effect = unit.action.effect.clone();
                    for (effect, vars, status_id, status_color) in
                        unit.all_statuses.iter().flat_map(|status| {
                            status.trigger(|trigger| matches!(trigger, StatusTrigger::Action))
                        })
                    {
                        self.effects.push_front(QueuedEffect {
                            effect,
                            context: EffectContext {
                                caster: Some(unit.id),
                                from: Some(unit.id),
                                target: Some(target.id),
                                vars,
                                status_id: Some(status_id),
                                color: Some(status_color),
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
                            color: None,
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
                self.model.time_scale = 1.0;
                self.model.acting_unit = Some(unit.id);
            }
            ActionState::Cooldown { time } => {
                self.model.current_tick.visual_timer += Time::new(UNIT_PRE_ACTION_TIME);
                self.model.acting_unit = Some(unit.id);
            }
            _ => {}
        }
    }
}
