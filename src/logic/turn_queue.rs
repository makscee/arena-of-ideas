use super::*;

impl Logic {
    pub fn process_turn_queue(&mut self) {
        if self.model.current_tick.visual_timer > Time::new(0.0)
            || self.model.lives <= 0
            || self.model.transition
        {
            return;
        }
        let (unit, state) = self
            .model
            .turn_queue
            .front_mut()
            .expect("Action queue is empty");
        let unit = self.model.units.remove(unit);
        if unit.is_none() {
            self.model.turn_queue.pop_front();
            self.model.acting_unit = None;
            return;
        }
        let mut unit = unit.unwrap();

        match state {
            TurnState::None => {
                *state = TurnState::PreTurn;
                self.model.current_tick.visual_timer += Time::new(UNIT_SWITCH_TIME);
            }
            TurnState::PreTurn => {
                self.model.acting_unit = Some(unit.id);
                *state = TurnState::Turn;
                self.model.current_tick.visual_timer += Time::new(UNIT_PRE_TURN_TIME);
                self.process_unit_statuses(&mut unit);
            }
            TurnState::Turn => {
                self.model.units.insert(unit.clone());
                let modifier_targets = self.collect_modifier_targets(&unit);
                self.model.units.remove(&unit.id);
                self.process_modifiers(&mut unit, &modifier_targets);
                unit.modifier_targets = modifier_targets;
                self.process_unit_targeting(&mut unit);

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
                                    status
                                        .trigger(|trigger| matches!(trigger, StatusTrigger::Action))
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
                        self.model.current_tick.visual_timer += Time::new(UNIT_TURN_TIME);
                    }
                    ActionState::Cooldown { time } => {}
                    _ => {}
                }
                self.tick_unit_cooldowns(&mut unit);

                self.model.time_scale = 1.0;
                self.model.turn_queue.pop_front();
            }
        }
        self.model.units.insert(unit);
    }
}
