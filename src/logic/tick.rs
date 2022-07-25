use geng::prelude::itertools::Itertools;

use super::*;

impl Logic {
    pub fn process_tick(&mut self) {
        if self.check_end() {
            self.model.transition = true;
        } else if self.effects.is_empty() && self.model.current_tick.visual_timer <= Time::new(0.0)
        {
            let last_tick = &self.model.current_tick;
            self.model.current_tick = TickModel::new(last_tick.tick_num + 1);
            self.process_units(Self::tick_unit_cooldowns);
        }
        self.model.current_tick.visual_timer -= self.delta_time;
    }
    fn tick_unit_cooldowns(&mut self, unit: &mut Unit) {
        if let ActionState::Cooldown { time } = &mut unit.action_state {
            *time += 1;
            if *time >= unit.action.cooldown {
                unit.action_state = ActionState::None;
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
