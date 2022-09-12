use geng::prelude::itertools::Itertools;

use super::*;

impl Logic {
    pub fn process_tick(&mut self) {
        if self.check_end() {
            let wounds: i32 = self
                .model
                .units
                .iter()
                .filter(|unit| unit.faction == Faction::Enemy)
                .map(|unit| unit.stats.attack)
                .sum();
            if self.model.lives <= 0 {
                return;
            }
            self.model.lives -= wounds;
            self.model.transition = self.model.lives > 0;
        } else if self.effects.is_empty()
            && self.model.current_tick.visual_timer <= Time::new(0.0)
            && self.model.current_tick.phase_timer <= Time::new(0.0)
        {
            self.model.time_scale = 1.0;
            let last_tick = &self.model.current_tick;
            self.model.current_tick = TickModel::new(last_tick.tick_num + 1);
        }
        self.model.current_tick.visual_timer -= self.delta_time;
        self.model.current_tick.phase_timer -= self.delta_time;
        self.model.current_tick.visual_timer = self.model.current_tick.visual_timer.max(r32(0.0));
        self.model.current_tick.phase_timer = self.model.current_tick.phase_timer.max(r32(0.0));
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
