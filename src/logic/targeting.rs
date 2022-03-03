use super::*;

impl Game {
    pub fn process_targeting(&mut self, unit: &mut Unit) {
        if unit
            .statuses
            .iter()
            .any(|status| matches!(status, Status::Freeze))
        {
            return;
        }
        if let AttackState::None = unit.attack_state {
            let target = match unit.target_ai {
                TargetAi::Closest => self
                    .units
                    .iter_mut()
                    .filter(|other| other.faction != unit.faction)
                    .min_by_key(|other| (other.position - unit.position).len()),
                _ => todo!(),
            };
            if let Some(target) = target {
                if distance_between_units(target, &unit) < unit.attack_radius {
                    unit.attack_state = AttackState::Start {
                        time: Time::new(0.0),
                        target: target.id,
                    }
                }
            }
        }
    }
}
