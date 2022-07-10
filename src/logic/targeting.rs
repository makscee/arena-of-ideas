use super::*;

impl Logic<'_> {
    pub fn process_targeting(&mut self) {
        self.process_units(Self::process_unit_targeting);
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
                    .filter(|other| other.faction != unit_faction)
                    .min_by_key(|other| (other.position.x - unit.position.x).abs()),
                TargetAi::Biggest => self
                    .model
                    .units
                    .iter()
                    .filter(|other| other.faction != unit_faction)
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
