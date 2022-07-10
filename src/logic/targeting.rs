use super::*;

impl Logic<'_> {
    pub fn process_targeting(&mut self) {
        self.model.current_tick.current_action_time_left -= self.delta_time;
        if self.model.current_tick.current_action_time_left <= Time::ZERO {
            if let Some(unit_id) = self.model.current_tick.action_queue.pop_front() {
                if let Some(unit) = self.model.units.get(&unit_id) {
                    let mut unit = unit.clone();
                    self.process_unit_targeting(&mut unit);
                    self.model.current_tick.current_action_time_left = (Time::new(TICK_TIME)
                        - self.model.current_tick.tick_time)
                        / Time::new((self.model.current_tick.action_queue.len() + 1) as f32);
                    self.model.units.insert(unit);
                }
            }
        }
    }
    fn process_unit_targeting(&mut self, unit: &mut Unit) {
        if unit
            .flags
            .iter()
            .any(|flag| matches!(flag, UnitStatFlag::ActionUnable))
        {
            return;
        }

        let unit_faction = unit.faction;
        if let ActionState::None = unit.action_state {
            let target = None;
            let target = target.or_else(|| match unit.target_ai {
                TargetAi::Closest => self
                    .model
                    .units
                    .iter()
                    .filter(|other| other.faction != unit_faction && other.position.height == 0)
                    .min_by_key(|other| (other.position.x - unit.position.x).abs()),
                TargetAi::Biggest => self
                    .model
                    .units
                    .iter()
                    .filter(|other| other.faction != unit_faction && other.position.height == 0)
                    .max_by_key(|other| other.stats.health),
                _ => todo!(),
            });
            if let Some(target) = target {
                if distance_between_units(target, &unit) < unit.action.range {
                    assert_ne!(target.id, unit.id);
                    unit.face_dir =
                        (target.position.to_world() - unit.position.to_world()).normalize_or_zero();
                    unit.action_state = ActionState::Start {
                        time: Time::new(0.0),
                        target: target.id,
                    }
                }
            }
        }
    }
}
