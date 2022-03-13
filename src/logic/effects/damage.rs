pub use super::*;

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

impl DamageEffect {
    pub fn walk_children_mut(&mut self, f: &mut impl FnMut(&mut Effect)) {
        for effect in self.on.values_mut() {
            effect.walk_mut(f);
        }
    }
}

impl Logic<'_> {
    pub fn process_damage_effect(
        &mut self,
        QueuedEffect {
            effect,
            caster,
            target,
        }: QueuedEffect<DamageEffect>,
    ) {
        let target_unit = target
            .and_then(|id| self.model.units.get_mut(&id))
            .expect("Target not found");
        let mut damage =
            target_unit.max_hp * effect.hp.relative / Health::new(100.0) + effect.hp.absolute;
        damage = min(damage, target_unit.hp);
        if damage > Health::new(0.0) {
            if let Some((index, _)) = target_unit
                .attached_statuses
                .iter()
                .enumerate()
                .find(|(_, status)| matches!(status, Status::Shield))
            {
                damage = Health::new(0.0);
                target_unit.attached_statuses.remove(index);
            } else if target_unit
                .all_statuses
                .iter()
                .any(|status| matches!(status, Status::Shield))
            {
                damage = Health::new(0.0);
            }
        }
        if damage > Health::new(0.0) {
            target_unit
                .attached_statuses
                .retain(|status| !matches!(status, Status::Freeze));

            for trigger in &target_unit.triggers {
                if let UnitTrigger::TakeDamage(trigger) = trigger {
                    if match &trigger.damage_type {
                        Some(damage_type) => effect.types.contains(damage_type),
                        None => true,
                    } {
                        self.effects.push_back(QueuedEffect {
                            caster,
                            target,
                            effect: trigger.effect.clone(),
                        });
                    }
                }
            }
        }
        let old_hp = target_unit.hp;
        target_unit.hp -= damage;
        if let Some(render) = &mut self.render {
            render.add_text(target_unit.position, &format!("{}", -damage), Color::RED);
        }
        let killed = old_hp > Health::new(0.0) && target_unit.hp <= Health::new(0.0);
        if killed {
            // self.render.add_text(target.position, "KILL", Color::RED);
            if let Some(effect) = effect.on.get(&DamageTrigger::Kill) {
                self.effects.push_back(QueuedEffect {
                    effect: effect.clone(),
                    caster,
                    target: Some(target_unit.id),
                });
            }
        }

        // Lifesteal
        let lifesteal =
            damage * effect.lifesteal.relative / Health::new(100.0) + effect.lifesteal.absolute;
        if let Some(caster) = caster.and_then(|id| self.model.units.get_mut(&id)) {
            caster.hp = (caster.hp + lifesteal).min(caster.max_hp);
        }
        if let Some(caster) = caster {
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
                                caster: Some(caster.id),
                                target,
                                effect: trigger.effect.clone(),
                            });
                        }
                    }
                }
            }
        }
    }
}
