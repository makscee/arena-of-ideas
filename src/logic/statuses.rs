use super::*;

impl Logic<'_> {
    pub fn process_statuses(&mut self) {
        let mut expired: Vec<(i64, String)> = Vec::new();
        for unit in &mut self.model.units {
            for status in &mut unit.all_statuses {
                if let Some(time) = &mut status.time {
                    *time -= self.delta_time;
                }
                for listener in &mut status.status.listeners {
                    let ticks = listener
                        .triggers
                        .iter_mut()
                        .map(|trigger| match trigger {
                            StatusTrigger::Repeating {
                                tick_time,
                                next_tick,
                            } => {
                                *next_tick -= self.delta_time;
                                let mut ticks = 0;
                                while *next_tick < Time::ZERO {
                                    ticks += 1;
                                    *next_tick += *tick_time;
                                }
                                ticks
                            }
                            _ => 0,
                        })
                        .sum();
                    for _ in 0..ticks {
                        self.effects.push_back(QueuedEffect {
                            effect: listener.effect.clone(),
                            context: EffectContext {
                                caster: status.caster,
                                from: None,
                                target: Some(unit.id),
                                vars: status.vars.clone(),
                            },
                        });
                    }
                }
            }

            expired.extend(
                (&mut unit.all_statuses)
                    .iter()
                    .filter(|x| {
                        if let Some(time) = x.time {
                            time <= R32::ZERO
                        } else {
                            false
                        }
                    })
                    .map(|status| (unit.id, status.status.name.clone())),
            );

            unit.all_statuses.retain(|status| {
                if let Some(time) = status.time {
                    if time <= R32::ZERO {
                        return false;
                    }
                }
                true
            });

            unit.flags = unit
                .all_statuses
                .iter()
                .flat_map(|status| status.status.flags.iter())
                .copied()
                .collect();
        }

        let mut auras: Vec<(Id, AuraStatus)> = Vec::new();
        for unit in &self.model.units {
            // for status in &unit.all_statuses {
            // TODO: reimplement
            // if let StatusOld::Aura(status) = &status.status {
            //     auras.push((unit.id, (**status).clone()));
            // }
            // }
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
                match &aura.clan {
                    Some(clan) => {
                        if !other.clans.contains(clan) {
                            continue;
                        }
                    }
                    _ => {}
                }
                // TODO: reimplement
                // other.all_statuses.push((*aura.status).clone());
            }
            self.model.units.insert(unit);
        }

        for (owner_id, status_name) in &expired {
            let owner = self.model.units.get(owner_id).unwrap();
            for (effect, vars) in owner.all_statuses.iter().flat_map(|status| {
                status.trigger(|trigger| match trigger {
                    StatusTrigger::SelfDetect {
                        status_name,
                        status_action,
                    } => {
                        status.status.name == *status_name && status_action == &StatusAction::Remove
                    }
                    _ => false,
                })
            }) {
                self.effects.push_front(QueuedEffect {
                    effect,
                    context: EffectContext {
                        caster: Some(owner.id),
                        from: Some(owner.id),
                        target: Some(owner.id),
                        vars,
                    },
                })
            }

            for other in &self.model.units {
                for (effect, vars) in other.all_statuses.iter().flat_map(|status| {
                    status.trigger(|trigger| match trigger {
                        StatusTrigger::Detect {
                            status_name,
                            filter,
                            status_action,
                        } => {
                            other.id != owner.id
                                && status.status.name == *status_name
                                && status_action == &StatusAction::Remove
                                && filter.matches(owner.faction, other.faction)
                        }
                        _ => false,
                    })
                }) {
                    self.effects.push_front(QueuedEffect {
                        effect,
                        context: EffectContext {
                            caster: Some(other.id),
                            from: Some(other.id),
                            target: Some(owner.id),
                            vars,
                        },
                    })
                }
            }
        }
    }
}
