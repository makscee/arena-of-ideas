use super::*;

impl Logic<'_> {
    pub fn process_targeting(&mut self) {
        self.process_units(Self::process_unit_targeting);
    }
    fn process_unit_targeting(&mut self, unit: &mut Unit) {
        if unit
            .all_statuses
            .iter()
            .any(|status| matches!(status, Status::Freeze | Status::Stun { .. }))
        {
            return;
        }
        if let AttackState::None = unit.attack_state {
            let target = match unit.target_ai {
                TargetAi::Closest => self
                    .model
                    .units
                    .iter_mut()
                    .filter(|other| other.faction != unit.faction)
                    .min_by_key(|other| (other.position - unit.position).len()),
                _ => todo!(),
            };
            if let Some(target) = target {
                if distance_between_units(target, &unit) < unit.attack.radius {
                    unit.attack_state = AttackState::Start {
                        time: Time::new(0.0),
                        target: target.id,
                    }
                }
            }
        }
    }
}
