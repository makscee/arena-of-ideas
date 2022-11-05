use geng::prelude::itertools::Itertools;

use super::*;

impl Logic {
    pub fn process_tick(&mut self) {
        if self.check_end() {
            if self.model.lives <= 0 {
                return;
            }
            if self.model.units.iter().any(|x| x.faction == Faction::Enemy) {
                self.model.lives -= 1;
            }
            self.model.transition = self.model.lives > 0;
            self.model.visual_timer += r32(1.0);
        } else if self.effects.is_empty()
            && self.model.visual_timer <= Time::new(0.0)
            && self.model.phase.timer <= Time::new(0.0)
        {
            self.model.time_scale = 1.0;
            let last_tick = &self.model.current_tick;
            self.model.current_tick = TickModel::new(last_tick.tick_num + 1);
        }
        if self.model.visual_timer > Time::ZERO {
            self.model.visual_timer -= self.delta_time;
        } else if self.model.phase.timer > Time::ZERO {
            self.model.phase.in_animation = true;
            self.model.phase.timer -= self.delta_time;
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
            && self.model.visual_timer <= Time::new(0.0)
    }
}
