use geng::prelude::itertools::Itertools;

use super::*;

impl Logic {
    pub fn process_tick(&mut self) {
        if self.check_end() {
            self.model.transition = true;
            if self
                .model
                .units
                .iter()
                .any(|unit| unit.faction == Faction::Enemy)
            {
                self.model.deaths += 1;
            }
        } else if self.effects.is_empty() && self.model.current_tick.visual_timer <= Time::new(0.0)
        {
            self.model.time_scale = 1.0;
            let last_tick = &self.model.current_tick;
            self.model.current_tick = TickModel::new(last_tick.tick_num + 1);
            self.tick();
        }
        self.model.current_tick.visual_timer -= self.delta_time;
    }
    fn tick(&mut self) {
        self.process_units(Self::tick_unit_cooldowns);
        self.tick_statuses();
    }
    fn tick_unit_cooldowns(&mut self, unit: &mut Unit) {
        if let ActionState::Cooldown { time } = &mut unit.action_state {
            if !unit
                .flags
                .iter()
                .any(|flag| matches!(flag, UnitStatFlag::ActionUnable))
            {
                *time += 1;
                if *time >= unit.cooldown {
                    unit.action_state = ActionState::None;
                }
            }
        }
    }
    fn check_end(&mut self) -> bool {
        self.model
            .units
            .iter()
            .unique_by(|unit| unit.faction)
            .count()
            < 2
            && self.effects.is_empty()
            && self.model.current_tick.visual_timer <= Time::new(0.0)
    }
}
