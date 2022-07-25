use super::*;

impl Logic {
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

        if let ActionState::None = unit.action_state {
            let unit_faction = unit.faction;
            let unit_id = unit.id;
            let target = self
                .model
                .units
                .iter()
                .filter(|other| {
                    other.id != unit.id
                        && other.faction != unit_faction
                        && other.position.height == 0
                        && distance_between_units(unit, other) < unit.action.range
                })
                .choose(&mut global_rng());

            if let Some(target) = target {
                unit.face_dir =
                    (target.position.to_world() - unit.position.to_world()).normalize_or_zero();
                unit.action_state = ActionState::Start { target: target.id }
            }
        }
    }
}
