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
    #[serde(default)]
    pub queue_delay: bool,
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
        let mut damage = effect.value.calculate(&context, &logic.model);
        let owner = logic.model.get_who(Who::Owner, &context);

        for (modifier_context, modifier_target) in &owner.modifier_targets {
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
                        if !logic.model.check_condition(&condition, &context) {
                            break;
                        }
                    }
                    damage = value.calculate(&context, &logic.model);
                }
                _ => (),
            }
        }

        let units = &mut logic.model.units;

        units.iter().for_each(|unit| {
            for (effect, trigger, mut vars, status_id, status_color) in
                unit.all_statuses.iter().flat_map(|status| {
                    status.trigger(|trigger| match trigger {
                        StatusTrigger::DamageHits {
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
                let context = EffectContext {
                    owner: unit.id,
                    creator: context.owner,
                    status_id: Some(status_id),
                    color: status_color,
                    ..context.clone()
                };
                logic.effects.push_back(context, effect);
            }
        });

        if damage <= 0 {
            return;
        }
        let dead_units = &mut logic.model.dead_units;
        let target_unit = logic.model.get_who(Who::Target, &context);

        for (effect, trigger, mut vars, status_id, status_color) in
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
            logic.effects.push_back(
                EffectContext {
                    owner: target_unit.id,
                    creator: context.owner,
                    vars: {
                        vars.insert(VarName::DamageIncoming, damage);
                        vars
                    },
                    status_id: Some(status_id),
                    color: status_color,
                    ..context.clone()
                },
                effect,
            )
        }

        let target_position = target_unit.position.clone();
        if target_unit
            .flags
            .iter()
            .any(|flag| matches!(flag, UnitStatFlag::DamageImmune))
        {
            logic.model.render_model.add_text(
                target_position,
                "ABSORBED",
                Rgba::RED,
                crate::render::TextType::Status,
            );
            return;
        }

        for status in target_unit.all_statuses.iter() {
            if status.status.name == "Vulnerability" {
                damage *= 2;
            }
        }

        if damage <= 0 {
            return;
        }

        for (effect, trigger, mut vars, status_id, status_color) in
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
            logic.effects.push_front(
                EffectContext {
                    owner: target_unit.id,
                    creator: context.owner,
                    vars: {
                        vars.insert(VarName::DamageIncoming, damage);
                        vars
                    },
                    status_id: Some(status_id),
                    color: status_color,
                    ..context.clone()
                },
                effect,
            )
        }

        let time = logic.model.time.clone();
        logic.model.render_model.add_text(
            target_position,
            &format!("{}", -damage),
            context.color,
            crate::render::TextType::Damage(effect.types.iter().cloned().collect()),
        );
        let target_unit = logic.model.get_who_mut(Who::Target, &context);
        let old_hp = target_unit.stats.health;
        target_unit.render.last_injure_time = time;
        target_unit.stats.health -= damage;
        target_unit.permanent_stats.health -= damage;
        let target_unit = logic.model.get_who(Who::Target, &context);
        let killed = old_hp > 0 && target_unit.stats.health <= 0;
        let owner_unit = logic.model.get_who(Who::Owner, &context);

        for (effect, trigger, mut vars, status_id, status_color) in
            owner_unit.all_statuses.iter().flat_map(|status| {
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
            logic.effects.push_back(
                EffectContext {
                    vars: {
                        vars.insert(VarName::DamageDealt, damage);
                        vars
                    },
                    status_id: Some(status_id),
                    color: status_color,
                    ..context.clone()
                },
                effect,
            )
        }

        if let Some(effect) = effect.on.get(&DamageTrigger::Injure) {
            logic.effects.push_back(
                {
                    let mut context = context.clone();
                    context.vars.insert(VarName::DamageDealt, damage);
                    context
                },
                effect.clone(),
            );
        }

        if killed {
            if let Some(effect) = effect.on.get(&DamageTrigger::Kill) {
                logic.effects.push_back(context.clone(), effect.clone());
            }
        }

        // Kill trigger

        if killed {
            for (effect, trigger, mut vars, status_id, status_color) in
                owner_unit.all_statuses.iter().flat_map(|status| {
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
                logic.effects.push_back(
                    EffectContext {
                        vars: {
                            vars.extend(context.vars.clone());
                            vars
                        },
                        status_id: Some(status_id),
                        color: status_color,
                        ..context.clone()
                    },
                    effect,
                )
            }
            logic.kill(context.target, context.clone());
        }

        let mut damage_instances = &mut logic.model.render_model.damage_instances;
        let avg_damage: i32 = damage_instances.iter().sum::<i32>() / damage_instances.len() as i32;
        if damage > avg_damage * 8 {
            logic.model.time_scale = 0.7;
        } else if damage > avg_damage * 3 {
            logic.model.time_scale = 0.9;
        }
        damage_instances.pop_front();
        damage_instances.push_back(damage);

        logic.model.render_model.damage_instances.pop_front();
        logic.model.render_model.damage_instances.push_back(damage);

        if effect.queue_delay {
            logic.effects.add_delay(&context, 1.0);
        }
    }
}
