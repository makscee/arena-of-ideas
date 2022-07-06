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
        // TODO: reimplement
        // if unit
        //     .all_statuses
        //     .iter()
        //     .any(|status| matches!(status.r#type(), StatusType::Freeze | StatusType::Stun))
        // {
        //     return;
        // }

        // TODO: reimplement
        // // This solution seems error-prone in case we forget to consider `Charmed` status at any point
        // // or use `unit.faction` instead of `unit_faction`
        // // The same code is used in the `ChangeTarget` effect
        // let unit_faction = unit
        //     .all_statuses
        //     .iter()
        //     .find_map(|status| match &status.status {
        //         StatusOld::Charmed(charm) => status
        //             .caster
        //             .and_then(|id| self.model.units.get(&id).map(|unit| unit.faction)),
        //         _ => None,
        //     })
        //     .unwrap_or(unit.faction);
        let unit_faction = unit.faction;

        if let ActionState::None = unit.action_state {
            // TODO: reimplement
            // // Priorities Taunt'ed enemies
            // let target = self
            //     .model
            //     .units
            //     .iter()
            //     .filter(|other| other.faction != unit_faction)
            //     .filter_map(|other| {
            //         other.all_statuses.iter().find_map(|status| match status {
            //             StatusOld::Taunt(status) => {
            //                 let distance = (other.position - unit.position).len();
            //                 if distance <= status.range {
            //                     Some((other, distance))
            //                 } else {
            //                     None
            //                 }
            //             }
            //             _ => None,
            //         })
            //     })
            //     .min_by_key(|(_, distance)| *distance)
            //     .map(|(unit, _)| unit);
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
                    unit.action_state = ActionState::Start {
                        time: Time::new(0.0),
                        target: target.id,
                    }
                }
            }
        }
    }
}
