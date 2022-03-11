use super::*;

impl Logic<'_> {
    pub fn spawn_unit(&mut self, unit_type: &UnitType, faction: Faction, position: Vec2<Coord>) {
        let template = &self.model.unit_templates[unit_type];
        let mut unit = Unit {
            id: self.model.next_id,
            unit_type: unit_type.clone(),
            spawn_animation_time_left: Some(template.spawn_animation_time),
            triggers: template.triggers.clone(),
            statuses: Vec::new(),
            faction,
            attack_state: AttackState::None,
            hp: template.hp,
            max_hp: template.hp,
            position,
            speed: template.speed,
            size: template.size,
            attack: template.attack.clone(),
            move_ai: template.move_ai,
            target_ai: template.target_ai,
            color: template.color,
            ability_cooldown: None,
            alliances: template.alliances.clone(),
        };
        self.model.next_id += 1;
        self.model.spawning_units.insert(unit);
    }
    pub fn process_spawns(&mut self) {
        let mut new_units = Vec::new();
        for unit in &mut self.model.spawning_units {
            if let Some(time) = &mut unit.spawn_animation_time_left {
                *time -= self.delta_time;
                if *time <= Time::new(0.0) {
                    unit.spawn_animation_time_left = None;
                    new_units.push(unit.clone());
                }
            }
        }
        for mut unit in new_units {
            for trigger in &unit.triggers {
                if let UnitTrigger::Spawn(effect) = trigger {
                    self.effects.push(QueuedEffect {
                        effect: effect.clone(),
                        caster: Some(unit.id),
                        target: Some(unit.id),
                    });
                }
            }
            self.model.units.insert(unit);
        }
        self.model
            .spawning_units
            .retain(|unit| unit.spawn_animation_time_left.is_some());
    }
}
