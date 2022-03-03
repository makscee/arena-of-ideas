use super::*;

impl Game {
    pub fn process_statuses(&mut self, unit: &mut Unit, delta_time: Time) {
        for status in &mut unit.statuses {
            match status {
                Status::Slow { time, .. } => {
                    *time -= delta_time;
                }
                Status::Freeze => {
                    unit.attack_state = AttackState::None;
                }
                _ => {}
            }
        }
        unit.statuses.retain(|status| match status {
            Status::Slow { time, .. } => *time > Time::ZERO,
            _ => true,
        });
    }
}
