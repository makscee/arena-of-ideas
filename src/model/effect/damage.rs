use super::*;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub enum DamageTrigger {
    Kill,
}

pub type DamageType = String;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct DamageEffect {
    pub hp: DamageValue,
    #[serde(default)]
    /// HP to heal self relative to the damage done
    pub lifesteal: DamageValue,
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
        let target_unit = context
            .target
            .and_then(|id| logic.model.units.get_mut(&id))
            .expect("Target not found");
        let mut damage = target_unit.max_hp * effect.hp.relative + effect.hp.absolute;
        damage = min(damage, target_unit.hp);
        if damage <= Health::new(0.0) {
            return;
        }

        // Invulnerability
        if target_unit
            .all_statuses
            .iter()
            .any(|status| matches!(status, Status::Invulnerability))
        {
            return;
        }

        // Shield
        if let Some(index) = target_unit
            .attached_statuses
            .iter()
            .position(|status| status.status.r#type() == StatusType::Shield)
        {
            for status in &target_unit.all_statuses {
                if let Status::ShieldBroken(UnitShieldBrokenTrigger { heal }) = *status {
                    let heal = heal.absolute + damage * heal.relative;
                    logic.effects.push_front(QueuedEffect {
                        context: EffectContext {
                            caster: None,
                            from: None,
                            target: Some(target_unit.id),
                        },
                        effect: Effect::Heal(Box::new(HealEffect {
                            hp: DamageValue::absolute(heal.as_f32()),
                            heal_past_max: DamageValue::ZERO,
                            max_hp: DamageValue::ZERO,
                        })),
                    });
                }
            }
            damage = Health::new(0.0);
            target_unit.attached_statuses.remove(index);
        } else if target_unit
            .all_statuses
            .iter()
            .any(|status| matches!(status, Status::Shield))
        {
            damage = Health::new(0.0);
        }
        if damage <= Health::new(0.0) {
            return;
        }

        // Freeze
        target_unit
            .attached_statuses
            .retain(|status| status.status.r#type() != StatusType::Freeze);

        for status in &target_unit.all_statuses {
            if let Status::Injured(trigger) = status {
                if match &trigger.damage_type {
                    Some(damage_type) => effect.types.contains(damage_type),
                    None => true,
                } {
                    logic.effects.push_front(QueuedEffect {
                        effect: trigger.effect.clone(),
                        context,
                    });
                }
            }
        }

        // Protection
        for status in &target_unit.all_statuses {
            if let Status::Protection { percent } = *status {
                damage *= r32(1.0 - percent / 100.0);
            }
        }
        if damage <= Health::new(0.0) {
            return;
        }

        let old_hp = target_unit.hp;
        target_unit.hp -= damage;
        let target_unit = logic.model.units.get(&context.target.unwrap()).unwrap();
        if let Some(render) = &mut logic.render {
            render.add_text(target_unit.position, &format!("{}", -damage), Color::RED);
        }
        let killed = old_hp > Health::new(0.0) && target_unit.hp <= Health::new(0.0);
        if killed {
            // logic.render.add_text(target.position, "KILL", Color::RED);
            if let Some(effect) = effect.on.get(&DamageTrigger::Kill) {
                logic.effects.push_front(QueuedEffect {
                    effect: effect.clone(),
                    context: EffectContext {
                        target: Some(target_unit.id),
                        ..context
                    },
                });
            }
        }

        // Lifesteal
        let lifesteal = damage * effect.lifesteal.relative + effect.lifesteal.absolute;
        if let Some(caster) = context.caster.and_then(|id| logic.model.units.get_mut(&id)) {
            caster.hp = (caster.hp + lifesteal).min(caster.max_hp);
        }
        if let Some(caster) = context.caster {
            let caster = logic
                .model
                .units
                .get(&caster)
                .or(logic.model.dead_units.get(&caster))
                .unwrap();
            if killed {
                for status in &caster.all_statuses {
                    if let Status::Kill(trigger) = status {
                        if match &trigger.damage_type {
                            Some(damage_type) => effect.types.contains(damage_type),
                            None => true,
                        } {
                            logic.effects.push_front(QueuedEffect {
                                effect: trigger.effect.clone(),
                                context,
                            });
                        }
                    }
                }
                logic.kill(context.target.unwrap());
            }
        }
    }
}
