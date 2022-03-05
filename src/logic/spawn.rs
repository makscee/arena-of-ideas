use super::*;

impl Game {
    pub fn spawn_unit(&mut self, template: &UnitTemplate, faction: Faction, position: Vec2<Coord>) {
        let mut unit = Unit {
            id: self.next_id,
            spawn_animation_time_left: Some(template.spawn_animation_time),
            spawn_effects: template.spawn_effects.clone(),
            statuses: Vec::new(),
            faction,
            attack_state: AttackState::None,
            hp: template.hp,
            max_hp: template.hp,
            position: position
                + vec2(
                    global_rng().gen_range(Coord::new(-1.0)..=Coord::new(1.0)),
                    global_rng().gen_range(Coord::new(-1.0)..=Coord::new(1.0)),
                ) * Coord::new(0.01),
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
        self.spawning_units.insert(unit);
    }
    pub fn process_spawns(&mut self) {
        let mut new_units = Vec::new();
        for unit in &mut self.spawning_units {
            if let Some(time) = &mut unit.spawn_animation_time_left {
                *time -= self.delta_time;
                if *time <= Time::new(0.0) {
                    unit.spawn_animation_time_left = None;
                    new_units.push(unit.clone());
                }
            }
        }
        for mut unit in new_units {
            for effect in &unit.spawn_effects.clone() {
                self.apply_effect(effect, None, &mut unit);
            }
            self.units.insert(unit);
        }
        self.spawning_units
            .retain(|unit| unit.spawn_animation_time_left.is_some());
    }
}
