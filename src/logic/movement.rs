use super::*;

impl Logic<'_> {
    pub fn process_movement(&mut self) {
        self.process_units(Self::process_unit_movement);
    }
    fn process_unit_movement(&mut self, unit: &mut Unit) {
        if unit
            .all_statuses
            .iter()
            .any(|status| matches!(status.r#type(), StatusType::Freeze | StatusType::Stun))
        {
            return;
        }
        if matches!(unit.action_state, ActionState::Start { .. }) {
            return;
        }
        let mut target_position = unit.position;
        match unit.move_ai {
            MoveAi::Advance => {
                let closest_enemy = self
                    .model
                    .units
                    .iter()
                    .filter(|other| other.faction != unit.faction)
                    .min_by_key(|other| (other.position - unit.position).len());
                if let Some(closest_enemy) = closest_enemy {
                    if distance_between_units(closest_enemy, &unit) > unit.action.radius {
                        target_position = closest_enemy.position;
                    }
                }
            }
            MoveAi::Avoid => {
                let closest_enemy = self
                    .model
                    .units
                    .iter()
                    .filter(|other| other.faction != unit.faction)
                    .min_by_key(|other| (other.position - unit.position).len());
                if let Some(closest_enemy) = closest_enemy {
                    if distance_between_units(closest_enemy, &unit) > unit.action.radius {
                        target_position = unit.position + (unit.position - closest_enemy.position);
                    }
                }
            }
            MoveAi::KeepClose => {
                // TODO: better implementation?
                let closest_ally = self
                    .model
                    .units
                    .iter()
                    .filter(|other| other.faction == unit.faction)
                    .min_by_key(|other| (other.position - unit.position).len());
                if let Some(closest_ally) = closest_ally {
                    target_position = closest_ally.position;
                }
            }
        }
        let mut speed = unit.speed;
        for status in &unit.all_statuses {
            match status {
                Status::Slow(status) => {
                    speed *= Coord::new(1.0 - status.percent / 100.0);
                }
                _ => {}
            }
        }
        unit.position += (target_position - unit.position).clamp_len(..=speed * self.delta_time);
    }
}
