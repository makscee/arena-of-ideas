use geng::prelude::itertools::Itertools;

use super::*;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub enum DamageTrigger {
    Injure,
    Kill,
}
pub const PURE_DAMAGE: &str = "Pure";
pub type DamageType = String;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct DamageEffect {
    pub value: Expr,
    #[serde(default)]
    pub types: HashSet<DamageType>,
    #[serde(default)]
    pub on: HashMap<DamageTrigger, Effect>,
}

impl EffectContainer for DamageEffect {
    fn walk_effects_mut(&mut self, f: &mut dyn FnMut(&mut Effect)) {
        for effect in self.on.values_mut() {
            effect.walk_mut(f);
        }
    }
}

impl EffectImpl for DamageEffect {
    fn process(self: Box<Self>, context: EffectContext, logic: &mut logic::Logic) {
        let mut effect = *self;
        let mut damage = effect.value.calculate(&context, logic);
        if let Some(caster) = context.caster {
            let caster_unit = logic
                .model
                .units
                .get(&caster)
                .or(logic.model.dead_units.get(&caster))
                .unwrap();
            for (modifier_context, modifier_target) in &caster_unit.modifier_targets {
                match modifier_target {
                    //Add extra damage types
                    ModifierTarget::ExtraOutDamageType {
                        source,
                        damage_type,
                    } => {
                        if effect
                            .types
                            .iter()
                            .any(|source_type| source.contains(source_type))
                        {
                            effect.types.extend(damage_type.clone());
                        }
                    }
                    //Modify damage value
                    ModifierTarget::Damage {
                        source,
                        condition,
                        value,
                    } => {
                        let mut context = context.clone();
                        context.vars.insert(VarName::DamageIncoming, damage);
                        context.vars.extend(modifier_context.vars.clone());
                        if let Some(damage_types) = source {
                            if !effect
                                .types
                                .iter()
                                .any(|source_type| damage_types.contains(source_type))
                            {
                                break;
                            }
                        }
                        if let Some(condition) = condition {
                            if !logic.check_condition(condition, &context) {
                                break;
                            }
                        }
                        damage = value.calculate(&context, logic);
                    }
                    _ => (),
                }
            }
        }

        let units = &mut logic.model.units;
        let dead_units = &mut logic.model.dead_units;
        let target_unit = context
            .target
            .and_then(|id| units.get_mut(&id).or(dead_units.get_mut(&id)))
            .expect("Target not found");

        if damage <= Health::new(0.0) {
            return;
        }

        for (effect, mut vars, status_id, status_color) in
            target_unit.all_statuses.iter().flat_map(|status| {
                status.trigger(|trigger| match trigger {
                    StatusTrigger::DamageIncoming {
                        damage_type,
                        except,
                    } => {
                        !effect.types.contains(&except.clone().unwrap_or_default())
                            && (damage_type.is_none()
                                || effect.types.contains(&damage_type.clone().unwrap()))
                    }
                    _ => false,
                })
            })
        {
            logic.effects.push_front(QueuedEffect {
                effect,
                context: EffectContext {
                    caster: context.caster,
                    from: context.from,
                    target: context.target,
                    vars: {
                        vars.insert(VarName::DamageIncoming, damage);
                        vars
                    },
                    status_id: Some(status_id),
                    color: Some(status_color),
                },
            })
        }

        if target_unit
            .flags
            .iter()
            .any(|flag| matches!(flag, UnitStatFlag::DamageImmune))
        {
            return;
        }

        // Block stat
        if !effect.types.contains(PURE_DAMAGE) && damage > r32(1.0) {
            let block = target_unit.stats.block;
            if block > r32(0.0) {
                damage = max(r32(1.0), damage - block);
            }
        }

        for status in target_unit.all_statuses.iter() {
            if status.status.name == "Vulnerability" {
                damage *= r32(2.0);
            }
        }

        for (effect, vars, status_id, status_color) in
            target_unit.all_statuses.iter().flat_map(|status| {
                status.trigger(|trigger| match trigger {
                    StatusTrigger::DamageTaken {
                        damage_type,
                        except,
                    } => {
                        !effect.types.contains(&except.clone().unwrap_or_default())
                            && (damage_type.is_none()
                                || effect.types.contains(&damage_type.clone().unwrap()))
                    }
                    _ => false,
                })
            })
        {
            logic.effects.push_front(QueuedEffect {
                effect,
                context: EffectContext {
                    caster: context.caster,
                    from: context.from,
                    target: context.target,
                    vars,
                    status_id: Some(status_id),
                    color: Some(status_color),
                },
            })
        }

        if damage <= Health::new(0.0) {
            return;
        }

        let old_hp = target_unit.stats.health;
        target_unit.last_injure_time = logic.model.time;
        target_unit.stats.health -= damage;
        target_unit.permanent_stats.health -= damage;
        let target_unit = logic
            .model
            .units
            .get(&context.target.unwrap())
            .or(logic.model.dead_units.get(&context.target.unwrap()))
            .unwrap();
        let damage_text = (damage * r32(10.0)).floor() / r32(10.0);
        let text_color = context.color.unwrap_or(Color::RED);
        logic.model.render_model.add_text(
            target_unit.position,
            &format!("{}", -damage_text),
            text_color,
            crate::render::TextType::Damage(effect.types.iter().cloned().collect()),
        );
        let killed = old_hp > Health::new(0.0) && target_unit.stats.health <= Health::new(0.0);

        if let Some(caster_unit) = context.caster.and_then(|id| logic.model.units.get(&id)) {
            for (effect, mut vars, status_id, status_color) in
                caster_unit.all_statuses.iter().flat_map(|status| {
                    status.trigger(|trigger| match trigger {
                        StatusTrigger::DamageDealt {
                            damage_type,
                            except,
                        } => {
                            !effect.types.contains(&except.clone().unwrap_or_default())
                                && (damage_type.is_none()
                                    || effect.types.contains(&damage_type.clone().unwrap()))
                        }
                        _ => false,
                    })
                })
            {
                logic.effects.push_front(QueuedEffect {
                    effect,
                    context: EffectContext {
                        caster: context.caster,
                        from: context.from,
                        target: context.target,
                        vars: {
                            vars.extend(context.vars.clone());
                            vars.insert(VarName::DamageDealt, damage);
                            vars
                        },
                        status_id: Some(status_id),
                        color: Some(status_color),
                    },
                })
            }
        }

        if let Some(effect) = effect.on.get(&DamageTrigger::Injure) {
            logic.effects.push_front(QueuedEffect {
                effect: effect.clone(),
                context: {
                    let mut context = context.clone();
                    context.vars.insert(VarName::DamageDealt, damage);
                    context
                },
            });
        }

        if killed {
            // logic.render.add_text(target.position, "KILL", Color::RED);
            if let Some(effect) = effect.on.get(&DamageTrigger::Kill) {
                logic.effects.push_front(QueuedEffect {
                    effect: effect.clone(),
                    context: context.clone(),
                });
            }
        }

        // Kill trigger
        if let Some(caster) = context.caster {
            let caster = logic
                .model
                .units
                .get(&caster)
                .or(logic.model.dead_units.get(&caster))
                .unwrap();
            if killed {
                for (effect, mut vars, status_id, status_color) in
                    caster.all_statuses.iter().flat_map(|status| {
                        status.trigger(|trigger| match trigger {
                            StatusTrigger::Kill {
                                damage_type,
                                except,
                            } => {
                                !effect.types.contains(&except.clone().unwrap_or_default())
                                    && (damage_type.is_none()
                                        || effect.types.contains(&damage_type.clone().unwrap()))
                            }
                            _ => false,
                        })
                    })
                {
                    logic.effects.push_front(QueuedEffect {
                        effect,
                        context: EffectContext {
                            caster: context.caster,
                            from: context.from,
                            target: context.target,
                            vars: {
                                vars.extend(context.vars.clone());
                                vars
                            },
                            status_id: Some(status_id),
                            color: Some(status_color),
                        },
                    })
                }
                logic.kill(context.target.unwrap());
            }
        }

        let mut damage_instances = &mut logic.model.damage_instances;
        let avg_damage: f32 = damage_instances.iter().sum::<f32>() / damage_instances.len() as f32;
        if damage.as_f32() > avg_damage * 4.0 {
            logic.model.time_scale = 0.3;
        } else if damage.as_f32() > avg_damage * 2.0 {
            logic.model.time_scale = 0.5;
        }
        damage_instances.pop_front();
        damage_instances.push_back(damage.as_f32());

        logic.model.damage_instances.pop_front();
        logic.model.damage_instances.push_back(damage.as_f32());
    }
}
