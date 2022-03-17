use super::*;

impl Logic<'_> {
    pub fn process_statuses(&mut self) {
        for unit in &mut self.model.units {
            for status in &mut unit.attached_statuses {
                match status {
                    Status::Slow { time, .. } => {
                        *time -= self.delta_time;
                    }
                    Status::Stun { time, .. } => {
                        *time -= self.delta_time;
                        unit.action_state = ActionState::None;
                    }
                    Status::Freeze => {
                        unit.action_state = ActionState::None;
                    }
                    Status::Aura(Aura { time, .. }) => {
                        if let Some(time) = time {
                            *time -= self.delta_time;
                        }
                    }
                    _ => {}
                }
            }
            unit.attached_statuses.retain(|status| match status {
                Status::Slow { time, .. } | Status::Stun { time, .. } => *time > Time::ZERO,
                Status::Aura(Aura {
                    time: Some(time), ..
                }) => *time > Time::ZERO,
                _ => true,
            });
        }
        for unit in &mut self.model.units {
            unit.all_statuses = unit.attached_statuses.clone();
        }

        let mut auras: Vec<(Id, Aura)> = Vec::new();
        for unit in &self.model.units {
            for status in &unit.attached_statuses {
                if let Status::Aura(aura) = status {
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
