use super::*;

const UNIT_STARTING_OFFSET: Vec2<f32> = vec2(0.5, 0.0);
const TIMER_PRE_STRIKE: f32 = 0.3;
const TIMER_POST_STRIKE: f32 = 0.05;
const TIMER_STRIKE: f32 = 0.02;
impl Logic {
    pub fn process_turn(&mut self) {
        if self.model.current_tick.phase_timer > Time::ZERO {
            if let Some(mut player) = self.model.units.remove(&self.model.current_tick.player) {
                self.process_turn_render_positions(&mut player);
                self.model.units.insert(player);
            }
            if let Some(mut enemy) = self.model.units.remove(&self.model.current_tick.enemy) {
                self.process_turn_render_positions(&mut enemy);
                self.model.units.insert(enemy);
            }
            return;
        }
        if self.model.current_tick.visual_timer > Time::ZERO
            || self.model.lives <= 0
            || self.model.transition
            || !self.effects.is_empty()
        {
            return;
        }
        match self.model.current_tick.turn_phase {
            TurnPhase::None => {
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
                    .expect("Cant find enemy unit")
                    .id;
                self.model.current_tick.turn_phase = TurnPhase::PreStrike;
                let timer = Time::new(TIMER_PRE_STRIKE);
                self.model.current_tick.phase_timer_start = timer;
                self.model.current_tick.phase_timer += timer;
            }
            TurnPhase::PreStrike => {
                self.model.current_tick.turn_phase = TurnPhase::Strike;
                let timer = Time::new(TIMER_STRIKE);
                self.model.current_tick.phase_timer += timer;
                self.model.current_tick.phase_timer_start = timer;
                self.process_units(Self::process_unit_statuses);
            }
            TurnPhase::Strike => {
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
                let timer = Time::new(TIMER_PRE_STRIKE);
                self.model.current_tick.phase_timer += timer;
                self.model.current_tick.phase_timer_start = timer;
                self.model.current_tick.turn_phase = TurnPhase::PostStrike;
            }
            TurnPhase::PostStrike => {
                let timer = Time::new(TIMER_PRE_STRIKE);
                self.model.current_tick.phase_timer += timer;
                self.model.current_tick.phase_timer_start = timer;
                self.model.current_tick.turn_phase = TurnPhase::None;
            }
        }
    }

    fn process_turn_render_positions(&mut self, unit: &mut Unit) {
        let phase_t = r32(1.0)
            - self.model.current_tick.phase_timer / self.model.current_tick.phase_timer_start;
        let unit_faction_factor = match unit.faction {
            Faction::Player => r32(-1.0),
            Faction::Enemy => r32(1.0),
        };
        let unit_pos = unit.position.to_world();
        let unit_starting_position =
            unit_pos + UNIT_STARTING_OFFSET.map(|x| r32(x) * unit_faction_factor);
        let unit_hit_position = vec2(unit_faction_factor * unit.render.radius, R32::ZERO);
        match self.model.current_tick.turn_phase {
            TurnPhase::PreStrike => {
                unit.render.render_position =
                    unit_pos + (unit_starting_position - unit_pos) * phase_t * phase_t;
            }
            TurnPhase::Strike => {
                unit.render.render_position =
                    unit_starting_position + (unit_hit_position - unit_starting_position) * phase_t;
            }
            TurnPhase::PostStrike => {
                unit.render.render_position =
                    unit_hit_position + (unit_pos - unit_hit_position) * phase_t;
            }
            _ => {}
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

        let mut effect = unit.action.clone();
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
