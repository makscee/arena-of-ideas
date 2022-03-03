use super::*;

impl Game {
    pub fn process_movement(&mut self, unit: &mut Unit, delta_time: Time) {
        if unit
            .statuses
            .iter()
            .any(|status| matches!(status, Status::Freeze))
        {
            return;
        }
        if matches!(unit.attack_state, AttackState::Start { .. }) {
            return;
        }
        let mut target_position = unit.position;
        match unit.move_ai {
            MoveAi::Advance => {
                let closest_enemy = self
                    .units
                    .iter()
                    .filter(|other| other.faction != unit.faction)
                    .min_by_key(|other| (other.position - unit.position).len());
                if let Some(closest_enemy) = closest_enemy {
                    if distance_between_units(closest_enemy, &unit) > unit.attack_radius {
                        target_position = closest_enemy.position;
                    }
                }
            }
            _ => todo!(),
        }
        let mut speed = unit.speed;
        for status in &unit.statuses {
            match status {
                Status::Slow { percent, .. } => {
                    speed *= Coord::new(1.0 - *percent / 100.0);
                }
                _ => {}
            }
        }
        unit.position += (target_position - unit.position).clamp_len(..=speed * delta_time);
    }
}
