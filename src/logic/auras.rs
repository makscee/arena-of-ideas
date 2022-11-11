use super::*;

impl Logic {
    pub fn process_auras(&mut self) {
        self.process_units(Self::process_unit_own_auras);
        self.process_units(Self::process_unit_received_auras);
    }

    fn process_unit_own_auras(&mut self, unit: &mut Unit) {
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
                                    other.id,
                                    unit.id,
                                )
                            })
                            .collect();
                        new_statuses.extend(statuses.iter().map(|status| {
                            (
                                other.id,
                                unit.id,
                                status.id,
                                status.status.name.clone(),
                                status.status.color,
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
        for (target, creator, status, name, color) in new_statuses {
            self.trigger_status_attach(
                EffectContext {
                    owner: target,
                    creator,
                    target,
                    queue_id: None,
                    vars: default(),
                    status_id: Some(status),
                    color,
                },
                status,
                &name,
            );
        }
    }

    fn process_unit_received_auras(&mut self, unit: &mut Unit) {
        let mut dropped_statuses = Vec::new();
        for i in 0..unit.all_statuses.len() {
            let status = unit.all_statuses.get(i).unwrap();
            if let Some(aura_id) = status.is_aura {
                let creator = status.creator;
                let keep = (|| {
                    let creator = self
                        .model
                        .units
                        .get(&creator)
                        .expect(&format!("Aura#{} creator#{} not found", aura_id, creator));
                    let aura = creator
                        .all_statuses
                        .iter()
                        .find(|status| status.id == aura_id)?;
                    let aura = match &aura.status.effect {
                        StatusEffect::Aura(aura) => aura,
                        _ => return None,
                    };
                    aura.is_applicable(creator, unit).then_some(())
                })()
                .is_some();
                let status = unit.all_statuses.get_mut(i).unwrap();
                if keep {
                    status.time = None;
                } else {
                    // The aura became inactive -> drop the status
                    status.time = Some(0);
                    unit.active_auras.remove(&aura_id);
                    dropped_statuses.push((status.creator, status.id, status.status.name.clone()));
                }
            }
        }
        unit.all_statuses
            .retain(|status| status.is_aura.is_none() || status.time.is_none());
        for (creator, status, name) in dropped_statuses {
            self.trigger_status_drop(UnitRef::Ref(unit), creator, status, &name);
        }
    }
}
