pub use super::*;

impl Logic<'_> {
    pub fn process_damage_effect(
        &mut self,
        QueuedEffect { effect, context }: QueuedEffect<DamageEffect>,
    ) {
        let target_unit = context
            .target
            .and_then(|id| self.model.units.get_mut(&id))
            .expect("Target not found");
        let mut damage =
            target_unit.max_hp * effect.hp.relative / Health::new(100.0) + effect.hp.absolute;
        damage = min(damage, target_unit.hp);
        if damage <= Health::new(0.0) {
            return;
        }

        // Shield
        if let Some(index) = target_unit
            .attached_statuses
            .iter()
            .position(|status| status.status.r#type() == StatusType::Shield)
        {
            for trigger in &target_unit.triggers {
                if let UnitTrigger::ShieldBroken(UnitShieldBrokenTrigger { heal }) = *trigger {
                    let heal = heal.absolute + damage * heal.relative;
                    self.effects.push_back(QueuedEffect {
                        context: EffectContext {
                            caster: None,
                            from: None,
                            target: Some(target_unit.id),
                        },
                        effect: Effect::Heal(Box::new(HealEffect {
                            hp: DamageValue::absolute(heal.as_f32()),
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

        for trigger in &target_unit.triggers {
            if let UnitTrigger::TakeDamage(trigger) = trigger {
                if match &trigger.damage_type {
                    Some(damage_type) => effect.types.contains(damage_type),
                    None => true,
                } {
                    self.effects.push_back(QueuedEffect {
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
        let target_unit = self.model.units.get(&context.target.unwrap()).unwrap();
        if let Some(render) = &mut self.render {
            render.add_text(target_unit.position, &format!("{}", -damage), Color::RED);
        }
        let killed = old_hp > Health::new(0.0) && target_unit.hp <= Health::new(0.0);
        if killed {
            // self.render.add_text(target.position, "KILL", Color::RED);
            if let Some(effect) = effect.on.get(&DamageTrigger::Kill) {
                self.effects.push_back(QueuedEffect {
                    effect: effect.clone(),
                    context: EffectContext {
                        target: Some(target_unit.id),
                        ..context
                    },
                });
            }
        }

        // Lifesteal
        let lifesteal =
            damage * effect.lifesteal.relative / Health::new(100.0) + effect.lifesteal.absolute;
        if let Some(caster) = context.caster.and_then(|id| self.model.units.get_mut(&id)) {
            caster.hp = (caster.hp + lifesteal).min(caster.max_hp);
        }
        if let Some(caster) = context.caster {
            let caster = self
                .model
                .units
                .get(&caster)
                .or(self.model.dead_units.get(&caster))
                .unwrap();
            if killed {
                for trigger in &caster.triggers {
                    if let UnitTrigger::Kill(trigger) = trigger {
                        if match &trigger.damage_type {
                            Some(damage_type) => effect.types.contains(damage_type),
                            None => true,
                        } {
                            self.effects.push_back(QueuedEffect {
                                effect: trigger.effect.clone(),
                                context,
                            });
                        }
                    }
                }
                self.kill(context.target.unwrap());
            }
        }
    }
}
