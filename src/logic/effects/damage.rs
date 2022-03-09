pub use super::*;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub enum DamageTrigger {
    Kill,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct DamageEffect {
    pub hp: DamageValue,
    #[serde(default)]
    /// HP to heal self relative to the damage done
    pub lifesteal: DamageValue,
    #[serde(default)]
    pub on: HashMap<DamageTrigger, Effect>,
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
        let target = target
            .and_then(|id| self.model.units.get_mut(&id))
            .expect("Target not found");
        let mut damage = match effect.hp {
            DamageValue::Absolute(hp) => hp,
            DamageValue::Relative(percent) => target.max_hp * percent / Health::new(100.0),
        };
        damage = min(damage, target.hp);
        if damage > Health::new(0.0) {
            if let Some((index, _)) = target
                .statuses
                .iter()
                .enumerate()
                .find(|(_, status)| matches!(status, Status::Shield))
            {
                damage = Health::new(0.0);
                target.statuses.remove(index);
            }
        }
        if damage > Health::new(0.0) {
            target
                .statuses
                .retain(|status| !matches!(status, Status::Freeze));
        }
        let old_hp = target.hp;
        target.hp -= damage;
        if let Some(render) = &mut self.render {
            render.add_text(target.position, &format!("-{}", damage), Color::RED);
        }
        if old_hp > Health::new(0.0) && target.hp <= Health::new(0.0) {
            // self.render.add_text(target.position, "KILL", Color::RED);
            if let Some(effect) = effect.on.get(&DamageTrigger::Kill) {
                self.effects.push(QueuedEffect {
                    effect: effect.clone(),
                    caster,
                    target: Some(target.id),
                });
            }
        }

        // Lifesteal
        let lifesteal = match effect.lifesteal {
            DamageValue::Absolute(hp) => hp,
            DamageValue::Relative(percent) => damage * percent / Health::new(100.0),
        };
        if let Some(caster) = caster.and_then(|id| self.model.units.get_mut(&id)) {
            caster.hp = (caster.hp + lifesteal).min(caster.max_hp);
        }
    }
}
