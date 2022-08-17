use geng::prelude::itertools::Itertools;

use super::*;

impl Logic {
    pub fn process_tick(&mut self) {
        self.model.current_tick.visual_timer -= self.delta_time;
        if self.check_end() {
            let wounds: i32 = self
                .model
                .units
                .iter()
                .filter(|unit| unit.faction == Faction::Enemy)
                .map(|unit| unit.stats.base_damage.ceil().as_f32() as i32)
                .sum();
            if self.model.lives <= 0 {
                return;
            }
            self.model.lives -= wounds;
            self.model.transition = self.model.lives > 0;
        } else if self.effects.is_empty()
            && self.model.current_tick.visual_timer <= Time::new(0.0)
            && self.model.turn_queue.is_empty()
        {
            self.model.time_scale = 1.0;
            let last_tick = &self.model.current_tick;
            self.model.current_tick = TickModel::new(last_tick.tick_num + 1);
            self.tick();
        }
    }
    fn tick(&mut self) {
        self.model.units.retain(|unit| !unit.is_dead);
        let mut units = self.model.units.iter().collect::<Vec<&Unit>>();

        let turn_queue: Vec<(Id, TurnState)> = units
            .into_iter()
            .sorted_by(|a, b| {
                Ord::cmp(
                    &(a.position.x.abs() - if a.faction == Faction::Player { 1 } else { 0 }),
                    &b.position.x.abs(),
                )
            })
            .map(|unit| (unit.id, TurnState::None))
            .collect();
        self.model.turn_queue.extend(turn_queue.into_iter());
    }
    pub fn tick_unit_cooldowns(&mut self, unit: &mut Unit) {
        if let ActionState::Cooldown { time } = &mut unit.action_state {
            if !unit
                .flags
                .iter()
                .any(|flag| matches!(flag, UnitStatFlag::ActionUnable))
            {
                *time += 1;
                if *time >= unit.stats.cooldown.floor().as_f32() as Ticks {
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
