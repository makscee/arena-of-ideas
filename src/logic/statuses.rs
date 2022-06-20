use super::*;

impl Logic<'_> {
    pub fn process_statuses(&mut self) {
        let mut expired: Vec<(Id, Id, String)> = Vec::new();

        fn is_expired(status: &AttachedStatus) -> bool {
            status.time.map(|time| time <= Time::ZERO).unwrap_or(false)
                || status
                    .vars
                    .get(&VarName::StackCounter)
                    .map(|count| *count <= R32::ZERO)
                    .unwrap_or(false)
        }

        for unit in &mut self.model.units {
            for status in &mut unit.all_statuses {
                if !status.is_inited {
                    for (effect, vars, status_id) in status.trigger(|trigger| match trigger {
                        StatusTrigger::Init => true,
                        _ => false,
                    }) {
                        self.effects.push_front(QueuedEffect {
                            effect,
                            context: EffectContext {
                                caster: Some(unit.id),
                                from: Some(unit.id),
                                target: Some(unit.id),
                                vars,
                                status_id: Some(status_id),
                            },
                        })
                    }
                    status.is_inited = true;
                }
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
                                from: Some(unit.id),
                                target: Some(unit.id),
                                vars: status.vars.clone(),
                                status_id: Some(status.id),
                            },
                        });
                    }
                }
                if is_expired(status) {
                    for (effect, vars, status_id) in status.trigger(|trigger| match trigger {
                        StatusTrigger::Break => true,
                        _ => false,
                    }) {
                        self.effects.push_front(QueuedEffect {
                            effect,
                            context: EffectContext {
                                caster: Some(unit.id),
                                from: Some(unit.id),
                                target: Some(unit.id),
                                vars,
                                status_id: Some(status_id),
                            },
                        })
                    }
                }
            }

            // Remember expired statuses
            expired.extend(
                (&mut unit.all_statuses)
                    .iter()
                    .filter(|status| is_expired(status) && !status.is_aura)
                    .map(|status| (unit.id, status.id, status.status.name.clone())),
            );

            unit.all_statuses.retain(|status| !is_expired(status));

            unit.flags = unit
                .all_statuses
                .iter()
                .flat_map(|status| status.status.flags.iter())
                .copied()
                .collect();
        }

        // Apply auras
        let auras: Vec<(Id, Aura)> = self
            .model
            .units
            .iter()
            .flat_map(|unit| {
                unit.all_statuses
                    .iter()
                    .flat_map(|status| &status.status.auras)
                    .map(|aura| (unit.id, aura.clone()))
            })
            .collect();
        for (caster_id, aura) in auras {
            let caster = self.model.units.remove(&caster_id).unwrap();
            for other in &mut self.model.units {
                match aura.radius {
                    Some(radius) => {
                        if distance_between_units(&caster, other) > radius {
                            continue;
                        }
                    }
                    _ => {}
                }
                if !aura.filter.check(other) {
                    continue;
                }
                let statuses: Vec<AttachedStatus> = aura
                    .statuses
                    .iter()
                    .filter_map(|status| {
                        match self.model.statuses.get(status) {
                            None => {
                                error!("Failed to find status: {status}");
                                None
                            }
                            Some(status) => {
                                let mut status =
                                    status.status.clone().attach_aura(Some(other.id), caster.id);
                                status.time = Some(R32::ZERO); // Force the status to be dropped next frame
                                Some(status)
                            }
                        }
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
            self.model.units.insert(caster);
        }

        // Detect expired statuses
        for (owner_id, detect_id, status_name) in &expired {
            let owner = self.model.units.get(owner_id).unwrap();
            for (effect, vars, status_id) in owner.all_statuses.iter().flat_map(|status| {
                status.trigger(|trigger| match trigger {
                    StatusTrigger::SelfDetect {
                        status_name,
                        status_action: StatusAction::Remove,
                    } => status.status.name == *status_name,
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
                        status_id: Some(*detect_id),
                    },
                })
            }

            for other in &self.model.units {
                for (effect, vars, status_id) in other.all_statuses.iter().flat_map(|status| {
                    status.trigger(|trigger| match trigger {
                        StatusTrigger::Detect {
                            status_name,
                            filter,
                            status_action: StatusAction::Remove,
                        } => {
                            other.id != owner.id
                                && status.status.name == *status_name
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
                            status_id: Some(*detect_id),
                        },
                    })
                }
            }
        }
    }
}
