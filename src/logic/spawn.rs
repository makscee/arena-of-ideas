use super::*;

impl Logic<'_> {
    pub fn spawn_unit(&mut self, unit_type: &UnitType, faction: Faction, position: Vec2<Coord>) {
        let mut template = self.model.unit_templates[unit_type].clone();
        self.spawn_template(unit_type, template, faction, position);
    }
    pub fn spawn_template(
        &mut self,
        unit_type: &UnitType,
        template: UnitTemplate,
        faction: Faction,
        position: Vec2<Coord>,
    ) {
        let mut unit = Unit {
            id: self.model.next_id,
            unit_type: unit_type.clone(),
            spawn_animation_time_left: Some(template.spawn_animation_time),
            attached_statuses: template.statuses.clone(),
            all_statuses: Vec::new(),
            faction,
            action_state: ActionState::None,
            hp: template.hp,
            max_hp: template.hp,
            position,
            speed: template.speed,
            size: template.size,
            action: template.action,
            move_ai: template.move_ai,
            target_ai: template.target_ai,
            render: template.render_mode.clone(),
            ability_cooldown: None,
            alliances: template.alliances,
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
            for status in &unit.all_statuses {
                if let Status::OnSpawn(status) = status {
                    self.effects.push_back(QueuedEffect {
                        effect: status.effect.clone(),
                        context: EffectContext {
                            caster: Some(unit.id),
                            from: Some(unit.id),
                            target: Some(unit.id),
                            vars: default(),
                        },
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
