use super::*;

impl Logic<'_> {
    pub fn process_movement(&mut self) {
        self.process_units(Self::process_unit_movement);
    }
    fn process_unit_movement(&mut self, unit: &mut Unit) {
        if unit
            .flags
            .iter()
            .any(|flag| matches!(flag, UnitStatFlag::MoveUnable))
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
                    if distance_between_units(closest_enemy, &unit) > unit.action.range {
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
                    if distance_between_units(closest_enemy, &unit) > unit.action.range {
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
            MoveAi::Stay => {
                target_position = unit.position;
            }
        }
        let mut speed = unit.speed;
        for status in &unit.all_statuses {
            match status {
                // TODO: reimplement
                // StatusOld::Slow(status) => {
                //     speed *= Coord::new(1.0 - status.percent / 100.0);
                // }
                _ => {}
            }
        }
        unit.position += (target_position - unit.position).clamp_len(..=speed * self.delta_time);

        if (target_position - unit.position).len().as_f32() > 0.0 {
            unit.face_dir = (target_position - unit.position).normalize_or_zero();
        }
    }
}
