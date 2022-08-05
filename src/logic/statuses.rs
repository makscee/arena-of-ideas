use geng::prelude::itertools::Itertools;

use super::*;

impl Logic {
    pub fn process_statuses(&mut self) {
        self.process_units(Self::process_unit_statuses);

        let ids: Vec<Id> = self.model.units.ids().copied().collect();
        for id in ids {
            let unit = self.model.units.get(&id).unwrap();
            let modifier_targets = self.collect_modifier_targets(unit);
            self.process_modifiers(&id, &modifier_targets);

            let mut unit_mut = self.model.units.get_mut(&id).unwrap();
            unit_mut.modifier_targets = modifier_targets;
        }
    }

    fn is_expired(status: &AttachedStatus) -> bool {
        !status.is_aura && status.time.map(|time| time <= Time::ZERO).unwrap_or(false)
            || status
                .vars
                .get(&VarName::StackCounter)
                .map(|count| *count <= R32::ZERO)
                .unwrap_or(false)
    }

    fn collect_modifier_targets(&self, unit: &Unit) -> Vec<(EffectContext, ModifierTarget)> {
        let mut modifier_targets: Vec<(EffectContext, ModifierTarget)> = Vec::new();
        for status in &unit.all_statuses {
            if !Self::is_expired(status) {
                if let StatusEffect::Modifier(modifier) = &status.status.effect {
                    let context = EffectContext {
                        caster: Some(unit.id),
                        from: status.caster,
                        target: Some(unit.id),
                        vars: status.vars.clone(),
                        status_id: Some(status.id),
                        color: None,
                    };
                    if let ModifierTarget::List { targets } = &modifier.target {
                        if self.check_condition(&modifier.condition, &context) {
                            modifier_targets.extend(
                                targets
                                    .iter()
                                    .map(|target| (context.clone(), target.clone())),
                            );
                        }
                    } else if self.check_condition(&modifier.condition, &context) {
                        modifier_targets.push((context.clone(), modifier.target.clone()));
                    }
                }
            }
        }
        modifier_targets
    }

    fn process_modifiers(
        &mut self,
        unit_id: &Id,
        modifier_targets: &Vec<(EffectContext, ModifierTarget)>,
    ) {
        let unit_mut = self.model.units.get_mut(unit_id).unwrap();
        unit_mut.stats = unit_mut.permanent_stats.clone();

        for (context, target) in modifier_targets {
            if let ModifierTarget::Stat { stat, value } = target {
                let stat_value = value.calculate(context, self);
                let mut unit_mut = self.model.units.get_mut(unit_id).unwrap();
                *unit_mut.stats.get_mut(*stat) = stat_value;
            }
        }
    }

    fn process_unit_statuses(&mut self, unit: &mut Unit) {
        let mut expired: Vec<(Option<Id>, Id, String)> = Vec::new();
        unit.all_statuses
            .sort_by(|a, b| a.status.order.cmp(&b.status.order));
        for status in &mut unit.all_statuses {
            if !status.is_inited {
                for (effect, vars, status_id, status_color) in
                    status.trigger(|trigger| matches!(trigger, StatusTrigger::Init))
                {
                    self.effects.push_front(QueuedEffect {
                        effect,
                        context: EffectContext {
                            caster: status.caster,
                            from: Some(unit.id),
                            target: Some(unit.id),
                            vars,
                            status_id: Some(status_id),
                            color: Some(status_color),
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
                            last_tick,
                        } => {
                            let cur_tick = r32(self.model.current_tick.tick_num as f32);
                            let mut ticks = 0;
                            if *last_tick == r32(0.0) {
                                if *next_tick == r32(0.0) {
                                    ticks += 1;
                                }
                                *next_tick = cur_tick + *tick_time;
                            } else if *next_tick == cur_tick {
                                ticks += 1;
                                *next_tick = cur_tick + *tick_time;
                            }
                            *last_tick = cur_tick;
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
                            color: Some(status.status.color),
                        },
                    });
                }
            }
            if Self::is_expired(status) {
                expired.push((status.caster, status.id, status.status.name.clone()));
                for (effect, vars, status_id, status_color) in
                    status.trigger(|trigger| matches!(trigger, StatusTrigger::Break))
                {
                    self.effects.push_front(QueuedEffect {
                        effect,
                        context: EffectContext {
                            caster: Some(unit.id),
                            from: Some(unit.id),
                            target: Some(unit.id),
                            vars,
                            status_id: Some(status_id),
                            color: Some(status_color),
                        },
                    })
                }
            } else if let StatusEffect::Aura(aura) = &status.status.effect {
                // Apply auras
                for other in &mut self.model.units {
                    if let Some(radius) = aura.radius {
                        // TODO: Check distance by util fn
                        if unit.position.distance(&other.position) > radius {
                            continue;
                        }
                    }
                    if !aura.filter.check(other) {
                        continue;
                    }
                    let statuses: Vec<AttachedStatus> = aura
                        .statuses
                        .iter()
                        .map(|status| {
                            let mut status = status
                                .get(&self.model.statuses)
                                .clone()
                                .attach_aura(Some(other.id), unit.id);
                            status.time = Some(R32::ZERO);
                            status
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

        unit.all_statuses.retain(|status| !Self::is_expired(status));

        unit.flags = unit
            .all_statuses
            .iter()
            .flat_map(|status| status.status.flags.iter())
            .copied()
            .collect();

        // Detect expired statuses
        for (caster_id, detect_id, detect_status) in &expired {
            for (effect, vars, status_id, status_color) in
                unit.all_statuses.iter().flat_map(|status| {
                    status.trigger(|trigger| match trigger {
                        StatusTrigger::SelfDetectAttach {
                            status_name,
                            status_action: StatusAction::Remove,
                        } => detect_status == status_name,
                        _ => false,
                    })
                })
            {
                self.effects.push_front(QueuedEffect {
                    effect,
                    context: EffectContext {
                        caster: *caster_id,
                        from: Some(unit.id),
                        target: Some(unit.id),
                        vars,
                        status_id: Some(*detect_id),
                        color: Some(status_color),
                    },
                })
            }

            for other in &self.model.units {
                for (effect, vars, status_id, status_color) in
                    other.all_statuses.iter().flat_map(|status| {
                        status.trigger(|trigger| match trigger {
                            StatusTrigger::DetectAttach {
                                status_name,
                                filter,
                                status_action: StatusAction::Remove,
                            } => {
                                other.id != unit.id
                                    && detect_status == status_name
                                    && filter.matches(unit.faction, other.faction)
                            }
                            _ => false,
                        })
                    })
                {
                    self.effects.push_front(QueuedEffect {
                        effect,
                        context: EffectContext {
                            caster: *caster_id,
                            from: Some(other.id),
                            target: Some(unit.id),
                            vars,
                            status_id: Some(*detect_id),
                            color: Some(status_color),
                        },
                    })
                }
            }
        }
    }
}
