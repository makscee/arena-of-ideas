use super::*;

impl Game {
    pub fn spawn_unit(&mut self, template: &UnitTemplate, faction: Faction, position: Vec2<Coord>) {
        let mut unit = Unit {
            id: self.next_id,
            statuses: Vec::new(),
            faction,
            attack_state: AttackState::None,
            hp: template.hp,
            max_hp: template.hp,
            position,
            speed: template.speed,
            projectile_speed: template.projectile_speed,
            attack_radius: template.attack_radius,
            size: template.size,
            attack_damage: template.attack_damage,
            attack_cooldown: template.attack_cooldown,
            attack_animation_delay: template.attack_animation_delay,
            attack_effects: template.attack_effects.clone(),
            kill_effects: template.kill_effects.clone(),
            death_effects: template.death_effects.clone(),
            move_ai: template.move_ai,
            target_ai: template.target_ai,
            color: template.color,
        };
        self.next_id += 1;
        for effect in &template.spawn_effects {
            self.apply_effect(effect, None, &mut unit);
        }
        self.units.insert(unit);
    }
}
