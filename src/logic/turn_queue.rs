use super::*;

impl Logic {
    pub fn process_turn(&mut self) {
        if self.model.current_tick.visual_timer > Time::new(0.0)
            || self.model.lives <= 0
            || self.model.transition
            || !self.effects.is_empty()
        {
            return;
        }
        match self.model.current_tick.turn_state {
            TurnState::None => {
                self.model.current_tick.player = self
                    .model
                    .units
                    .iter()
                    .find(|unit| unit.position.side == Faction::Player && unit.position.x == 0)
                    .expect("Cant find player unit")
                    .id;

                self.model.current_tick.enemy = self
                    .model
                    .units
                    .iter()
                    .find(|unit| unit.position.side == Faction::Enemy && unit.position.x == 0)
                    .expect("Cant find player unit")
                    .id;
                self.model.current_tick.turn_state = TurnState::PreTurn;
                self.model.current_tick.visual_timer += Time::new(UNIT_SWITCH_TIME);
            }
            TurnState::PreTurn => {
                self.model.current_tick.turn_state = TurnState::Turn;
                self.model.current_tick.visual_timer += Time::new(UNIT_PRE_TURN_TIME);
                self.process_units(Self::process_unit_statuses);
            }
            TurnState::Turn => {
                self.process_units(Self::process_modifiers);
                let player = self
                    .model
                    .units
                    .remove(&self.model.current_tick.player)
                    .expect("Cant find player unit");
                let enemy = self
                    .model
                    .units
                    .remove(&self.model.current_tick.enemy)
                    .expect("Cant find enemy unit");

                self.process_action(&player, &enemy);
                self.process_action(&enemy, &player);
                self.model.units.insert(player);
                self.model.units.insert(enemy);
                self.model.current_tick.visual_timer += Time::new(UNIT_TURN_TIME);
                self.model.current_tick.turn_state = TurnState::None;
            }
        }
    }

    fn process_action(&mut self, unit: &Unit, target: &Unit) {
        if unit
            .flags
            .iter()
            .any(|flag| matches!(flag, UnitStatFlag::ActionUnable))
        {
            return;
        }

        let mut effect = unit.action.effect.clone();
        for (effect, trigger, vars, status_id, status_color) in
            unit.all_statuses.iter().flat_map(|status| {
                status.trigger(|trigger| matches!(trigger, StatusTriggerType::Action))
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
        match unit.faction {
            Faction::Player => self.model.render_model.last_player_action_time = self.model.time,
            Faction::Enemy => self.model.render_model.last_enemy_action_time = self.model.time,
        }
    }
}
