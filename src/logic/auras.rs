use super::*;

impl Logic {
    pub fn process_auras(&mut self) {
        self.process_units(Self::process_unit_auras);
    }
    fn process_unit_auras(&mut self, unit: &mut Unit) {
        let mut dropped_statuses = Vec::new();
        for aura_status in &unit.all_statuses {
            if let StatusEffect::Aura(aura) = &aura_status.status.effect {
                // Apply auras
                for other in &mut self.model.units {
                    let applicable = (|| {
                        if let Some(radius) = aura.radius {
                            if unit.position.distance(&other.position) > radius {
                                return false;
                            }
                        }
                        aura.filter.check(other)
                    })();
                    if !applicable {
                        if other.active_auras.remove(&aura_status.id) {
                            // The aura became inactive for `other` -> drop the status
                            other.all_statuses.retain(|status| {
                                let should_drop = status.is_aura == Some(aura_status.id);
                                if should_drop {
                                    dropped_statuses.push((
                                        other.id,
                                        status.caster,
                                        status.id,
                                        status.status.name.clone(),
                                    ));
                                }
                                should_drop
                            });
                        }
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
        for (unit_id, caster, status, name) in dropped_statuses {
            self.trigger_status_drop(UnitRef::Id(unit_id), caster, status, &name);
        }
    }
}
