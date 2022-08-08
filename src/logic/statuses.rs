use geng::prelude::itertools::Itertools;

use super::*;

impl Logic {
    pub fn tick_statuses(&mut self) {
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

    fn is_status_expired(status: &AttachedStatus) -> bool {
        status.time.map(|time| time == 0).unwrap_or(false)
            || status
                .vars
                .get(&VarName::StackCounter)
                .map(|count| *count <= R32::ZERO)
                .unwrap_or(false)
    }

    fn collect_modifier_targets(&self, unit: &Unit) -> Vec<(EffectContext, ModifierTarget)> {
        let mut modifier_targets: Vec<(EffectContext, ModifierTarget)> = Vec::new();
        for status in &unit.all_statuses {
            if !Self::is_status_expired(status) {
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
                        if match &modifier.condition {
                            Some(condition) => self.check_condition(condition, &context),
                            None => true,
                        } {
                            modifier_targets.extend(
                                targets
                                    .iter()
                                    .map(|target| (context.clone(), target.clone())),
                            );
                        }
                    } else if match &modifier.condition {
                        Some(condition) => self.check_condition(condition, &context),
                        None => true,
                    } {
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
        let statuses_len = unit.all_statuses.len();
        for i in 0..statuses_len {
            let status = unit.all_statuses.get_mut(i).unwrap();
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
                *time = time.saturating_sub(1);
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
                            let cur_tick = self.model.current_tick.tick_num;
                            if cur_tick == *last_tick {
                                return 0;
                            };
                            let mut ticks = 0;
                            if *last_tick == 0 {
                                if *next_tick == 0 {
                                    ticks += 1;
                                }
                                *next_tick += cur_tick;
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
                            caster: status.caster.or(Some(unit.id)),
                            from: Some(unit.id),
                            target: Some(unit.id),
                            vars: status.vars.clone(),
                            status_id: Some(status.id),
                            color: Some(status.status.color),
                        },
                    });
                }
            }
            // Update aura info
            if let Some(aura_id) = status.is_aura {
                let caster = status.caster;
                let keep = (|| {
                    let caster = self.model.units.get(&caster?)?;
                    let aura = caster
                        .all_statuses
                        .iter()
                        .find(|status| status.id == aura_id)?;
                    let aura = match &aura.status.effect {
                        StatusEffect::Aura(aura) => aura,
                        _ => return None,
                    };
                    aura.is_applicable(caster, unit).then_some(())
                })()
                .is_some();
                let status = unit.all_statuses.get_mut(i).unwrap();
                if keep {
                    status.time = None;
                } else {
                    // The aura became inactive -> drop the status
                    status.time = Some(0);
                    unit.active_auras.remove(&aura_id);
                }
            }
            let status = unit.all_statuses.get_mut(i).unwrap();
            if Self::is_status_expired(status) {
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
            }
        }

        unit.all_statuses
            .retain(|status| !Self::is_status_expired(status));

        unit.flags = unit
            .all_statuses
            .iter()
            .flat_map(|status| status.status.flags.iter())
            .copied()
            .collect();

        // Detect expired statuses
        for (caster_id, status_id, detect_status) in expired {
            self.trigger_status_drop(UnitRef::Ref(unit), caster_id, status_id, &detect_status)
        }
    }

    pub(super) fn trigger_status_drop(
        &mut self,
        unit: UnitRef<'_>,
        caster_id: Option<Id>,
        status_id: Id,
        status_name: &StatusName,
    ) {
        let unit = match unit {
            UnitRef::Id(id) => self
                .model
                .units
                .get(&id)
                .expect("Failed to find unit by id"),
            UnitRef::Ref(unit) => unit,
        };
        for (effect, vars, status_id, status_color) in unit.all_statuses.iter().flat_map(|status| {
            status.trigger(|trigger| match trigger {
                StatusTrigger::SelfDetectAttach {
                    status_name: name,
                    status_action: StatusAction::Remove,
                } => status_name == name,
                _ => false,
            })
        }) {
            self.effects.push_front(QueuedEffect {
                effect,
                context: EffectContext {
                    caster: caster_id,
                    from: Some(unit.id),
                    target: Some(unit.id),
                    vars,
                    status_id: Some(status_id),
                    color: Some(status_color),
                },
            })
        }

        for other in &self.model.units {
            for (effect, vars, status_id, status_color) in
                other.all_statuses.iter().flat_map(|status| {
                    status.trigger(|trigger| match trigger {
                        StatusTrigger::DetectAttach {
                            status_name: name,
                            filter,
                            status_action: StatusAction::Remove,
                        } => {
                            other.id != unit.id
                                && status_name == name
                                && filter.matches(unit.faction, other.faction)
                        }
                        _ => false,
                    })
                })
            {
                self.effects.push_front(QueuedEffect {
                    effect,
                    context: EffectContext {
                        caster: caster_id,
                        from: Some(other.id),
                        target: Some(unit.id),
                        vars,
                        status_id: Some(status_id),
                        color: Some(status_color),
                    },
                })
            }
        }
    }
}
