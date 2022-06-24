use super::*;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub enum DamageTrigger {
    Injure,
    Kill,
}

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
        let effect = *self;
        let mut damage = effect.value.calculate(&context, logic);
        let armor_penetration = 0_f32;
        if let Some(caster) = context.caster {
            let armor_penetration = logic
                .model
                .units
                .get(&caster)
                .or(logic.model.dead_units.get(&caster))
                .unwrap()
                .armor_penetration;
        }
        let units = &mut logic.model.units;
        let target_unit = context
            .target
            .and_then(|id| units.get_mut(&id))
            .expect("Target not found");

        if damage <= Health::new(0.0) {
            return;
        }

        for (effect, mut vars, status_id) in target_unit.all_statuses.iter().flat_map(|status| {
            status.trigger(|trigger| match trigger {
                StatusTrigger::DamageIncoming { damage_type } => match &damage_type {
                    Some(damage_type) => effect.types.contains(damage_type),
                    None => true,
                },
                _ => false,
            })
        }) {
            logic.effects.push_front(QueuedEffect {
                effect,
                context: EffectContext {
                    caster: Some(target_unit.id),
                    from: context.from,
                    target: context.target,
                    vars: {
                        vars.insert(VarName::DamageIncoming, damage);
                        vars
                    },
                    status_id: Some(status_id),
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

        // Armor stat
        let armor = target_unit.armor.as_f32() - armor_penetration;
        if armor > 0.0 {
            damage *= r32(1.0 - (0.06 * armor) / (1.0 + 0.06 * armor));
        } else if armor < 0.0 {
            damage *= r32(2.0 - 0.94_f32.powf(-armor));
        }

        for status in target_unit.all_statuses.iter() {
            if status.status.name == "Vulnerability" {
                damage *= r32(2.0);
            }
        }

        for (effect, vars, status_id) in target_unit.all_statuses.iter().flat_map(|status| {
            status.trigger(|trigger| match trigger {
                StatusTrigger::DamageTaken { damage_type } => match &damage_type {
                    Some(damage_type) => effect.types.contains(damage_type),
                    None => true,
                },
                _ => false,
            })
        }) {
            logic.effects.push_front(QueuedEffect {
                effect,
                context: EffectContext {
                    caster: Some(target_unit.id),
                    from: context.from,
                    target: context.target,
                    vars,
                    status_id: Some(status_id),
                },
            })
        }

        // TODO: reimplement
        // // Protection
        // for status in &target_unit.all_statuses {
        //     if let StatusOld::Protection(status) = status {
        //         damage *= r32(1.0 - status.percent / 100.0);
        //     }
        // }
        if damage <= Health::new(0.0) {
            return;
        }

        let old_hp = target_unit.health;
        target_unit.last_injure_time = logic.model.time;
        target_unit.health -= damage;
        let target_unit = logic.model.units.get(&context.target.unwrap()).unwrap();
        if let Some(render) = &mut logic.render {
            let damage_text = (damage * r32(10.0)).floor() / r32(10.0);
            render.add_text(
                target_unit.position,
                &format!("{}", -damage_text),
                Color::RED,
            );
        }
        let killed = old_hp > Health::new(0.0) && target_unit.health <= Health::new(0.0);

        if let Some(caster_unit) = context.caster.and_then(|id| logic.model.units.get(&id)) {
            for (effect, mut vars, status_id) in
                caster_unit.all_statuses.iter().flat_map(|status| {
                    status.trigger(|trigger| match trigger {
                        StatusTrigger::DamageDealt { damage_type } => match damage_type {
                            Some(damage_type) => effect.types.contains(damage_type),
                            None => true,
                        },
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
                for (effect, mut vars, status_id) in caster.all_statuses.iter().flat_map(|status| {
                    status.trigger(|trigger| match trigger {
                        StatusTrigger::Kill { damage_type } => match damage_type {
                            Some(damage_type) => effect.types.contains(damage_type),
                            None => true,
                        },
                        _ => false,
                    })
                }) {
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
                        },
                    })
                }
                logic.kill(context.target.unwrap());
            }
        }
    }
}
