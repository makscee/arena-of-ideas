pub use super::*;

impl Game {
    pub fn process_damage_effect(
        &mut self,
        QueuedEffect {
            effect,
            caster,
            target,
        }: QueuedEffect<DamageEffect>,
    ) {
        let target = target
            .and_then(|id| self.units.get_mut(&id))
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
        self.render
            .add_text(target.position, &format!("-{}", damage), Color::RED);
        if old_hp > Health::new(0.0) && target.hp <= Health::new(0.0) {
            // self.render.add_text(target.position, "KILL", Color::RED);
            for kill_effect in effect.kill_effects {
                self.effects.push(QueuedEffect {
                    effect: kill_effect.clone(),
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
        if let Some(caster) = caster.and_then(|id| self.units.get_mut(&id)) {
            caster.hp = (caster.hp + lifesteal).min(caster.max_hp);
        }
    }
}
