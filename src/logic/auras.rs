use super::*;

impl Logic {
    pub fn process_auras(&mut self) {
        self.process_units(Self::process_unit_auras);
    }
    fn process_unit_auras(&mut self, unit: &mut Unit) {
        let mut dropped_statuses = Vec::new();
        let mut new_statuses = Vec::new();
        for aura_status in &unit.all_statuses {
            if let StatusEffect::Aura(aura) = &aura_status.status.effect {
                // Apply auras
                for other in &mut self.model.units {
                    if !aura.is_applicable(unit, other) {
                        continue;
                    }
                    if other.active_auras.insert(aura_status.id) {
                        // New aura
                        let statuses: Vec<AttachedStatus> = aura
                            .statuses
                            .iter()
                            .map(|status| {
                                status.get(&self.model.statuses).clone().attach_aura(
                                    aura_status.id,
                                    Some(other.id),
                                    unit.id,
                                )
                            })
                            .collect();
                        new_statuses.extend(statuses.iter().map(|status| {
                            (
                                other.id,
                                Some(unit.id),
                                status.id,
                                status.status.name.clone(),
                            )
                        }));
                        other.flags.extend(
                            statuses
                                .iter()
                                .flat_map(|status| status.status.flags.iter())
                                .copied(),
                        );
                        other.all_statuses.extend(statuses);
                    }
                }
            }
        }
        for (target, caster, status, name) in new_statuses {
            self.trigger_status_attach(target, caster, status, &name);
        }
        for (unit_id, caster, status, name) in dropped_statuses {
            self.trigger_status_drop(UnitRef::Id(unit_id), caster, status, &name);
        }
    }
}
