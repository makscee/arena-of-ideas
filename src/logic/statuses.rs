use super::*;

impl Game {
    pub fn process_statuses(&mut self) {
        self.process_units(Self::process_unit_statuses);
    }
    fn process_unit_statuses(&mut self, unit: &mut Unit) {
        for status in &mut unit.statuses {
            match status {
                Status::Slow { time, .. } => {
                    *time -= self.delta_time;
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
