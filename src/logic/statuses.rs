use super::*;

impl Logic<'_> {
    pub fn process_statuses(&mut self) {
        for unit in &mut self.model.units {
            for status in &mut unit.attached_statuses {
                if let Some(time) = &mut status.time {
                    *time -= self.delta_time;
                }
                if let StatusType::Freeze | StatusType::Stun = status.status.r#type() {
                    unit.action_state = ActionState::None;
                }
            }
            unit.attached_statuses.retain(|status| {
                if let Some(time) = status.time {
                    if time <= R32::ZERO {
                        return false;
                    }
                }
                true
            });
        }
        for unit in &mut self.model.units {
            unit.all_statuses = unit
                .attached_statuses
                .iter()
                .map(|status| status.status.clone())
                .collect();
        }

        let mut auras: Vec<(Id, Aura)> = Vec::new();
        for unit in &self.model.units {
            for status in &unit.attached_statuses {
                if let Status::Aura(aura) = &status.status {
                    auras.push((unit.id, aura.clone()));
                }
            }
        }
        for (unit_id, aura) in auras {
            let unit = self.model.units.remove(&unit_id).unwrap();
            for other in &mut self.model.units {
                if other.faction != unit.faction {
                    continue;
                }
                match aura.distance {
                    Some(distance) => {
                        if distance_between_units(&unit, other) > distance {
                            continue;
                        }
                    }
                    _ => {}
                }
                match &aura.alliance {
                    Some(alliance) => {
                        if !other.alliances.contains(alliance) {
                            continue;
                        }
                    }
                    _ => {}
                }
                other.all_statuses.push((*aura.status).clone());
            }
            self.model.units.insert(unit);
        }
    }
}
