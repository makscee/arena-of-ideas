use super::*;

impl Logic<'_> {
    pub fn process_targeting(&mut self) {
        self.process_units(Self::process_unit_targeting);
    }
    fn process_unit_targeting(&mut self, unit: &mut Unit) {
        if unit
            .all_statuses
            .iter()
            .any(|status| matches!(status.r#type(), StatusType::Freeze | StatusType::Stun))
        {
            return;
        }
        if let ActionState::None = unit.action_state {
            // Priorities Taunt'ed enemies
            let target = self
                .model
                .units
                .iter()
                .filter(|other| other.faction != unit.faction)
                .filter_map(|other| {
                    other.all_statuses.iter().find_map(|status| match status {
                        Status::Taunt(status) => {
                            let distance = (other.position - unit.position).len();
                            if distance <= status.range {
                                Some((other, distance))
                            } else {
                                None
                            }
                        }
                        _ => None,
                    })
                })
                .min_by_key(|(_, distance)| *distance)
                .map(|(unit, _)| unit);
            let target = target.or_else(|| match unit.target_ai {
                TargetAi::Closest => self
                    .model
                    .units
                    .iter()
                    .filter(|other| other.faction != unit.faction)
                    .min_by_key(|other| (other.position - unit.position).len()),
                TargetAi::Biggest => self
                    .model
                    .units
                    .iter()
                    .filter(|other| other.faction != unit.faction)
                    .max_by_key(|other| other.health),
                _ => todo!(),
            });
            if let Some(target) = target {
                if distance_between_units(target, &unit) < unit.action.range {
                    assert_ne!(target.id, unit.id);
                    unit.action_state = ActionState::Start {
                        time: Time::new(0.0),
                        target: target.id,
                    }
                }
            }
        }
    }
}
