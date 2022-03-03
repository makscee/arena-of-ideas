use super::*;

impl Game {
    pub fn deal_damage(
        &mut self,
        mut attacker: Option<&mut Unit>,
        target: &mut Unit,
        effects: &[Effect],
        kill_effects: &[Effect],
        mut damage: Health,
    ) {
        damage = min(damage, target.hp);
        if damage != 0 {
            if let Some((index, _)) = target
                .statuses
                .iter()
                .enumerate()
                .find(|(_, status)| matches!(status, Status::Shield))
            {
                damage = 0;
                target.statuses.remove(index);
            }
        }
        if damage != 0 {
            target
                .statuses
                .retain(|status| !matches!(status, Status::Freeze));
        }
        let old_hp = target.hp;
        target.hp -= damage;
        for effect in effects {
            self.apply_effect(effect, attacker.as_deref_mut(), target);
        }
        if old_hp > 0 && target.hp <= 0 {
            for effect in kill_effects {
                self.apply_effect(effect, attacker.as_deref_mut(), target);
            }
        }
    }
}
