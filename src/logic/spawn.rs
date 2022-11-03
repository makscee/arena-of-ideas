use super::*;

impl Logic {
    /// Spawns the unit and returns its id. If there is a unit in that position and there is an
    /// empty slot to the left, it and all units to the left are shifted to the left.
    /// Otherwise, if all slots are occupied, the unit is placed on top the unit in that position.
    pub fn spawn_by_type(&mut self, unit_type: &UnitType, position: Position) -> Id {
        let mut template = &self
            .model
            .unit_templates
            .get(unit_type)
            .unwrap_or_else(|| panic!("Failed to find unit template for {unit_type}"));

        let mut unit = Unit::new(
            &template,
            self.model.next_id,
            position,
            &self.model.statuses,
        );
        self.spawn_by_unit(unit)
    }

    pub fn spawn_by_unit(&mut self, mut unit: Unit) -> Id {
        let id = self.model.next_id;
        let position = unit.position;
        // Check empty slots
        // Shift the units, assuming that there are no empty slots in between
        self.model
            .units
            .iter_mut()
            .filter(|unit| unit.position.side == position.side && unit.position.x >= position.x)
            .for_each(|unit| unit.position.x += 1);
        unit.id = id;
        for (clan, _) in &self.model.clan_effects.map {
            let mut size = 0;
            match unit.faction {
                Faction::Player => {
                    if let Some(members) = self.model.config.clans.get(&clan) {
                        size = *members;
                    }
                }
                Faction::Enemy => {
                    if let Some(members) = self.model.config.clans.get(&clan) {
                        size = *members;
                    } else if let Some(members) = self.model.config.enemy_clans.get(&clan) {
                        size = *members;
                    }
                }
            }

            clan.apply_effects(
                &mut unit,
                &self.model.clan_effects,
                size,
                self.model.next_id,
                &self.model.statuses,
            );
        }

        // On spawn effects
        for (effect, trigger, vars, status_id, status_color) in
            unit.all_statuses.iter().flat_map(|status| {
                status.trigger(|trigger| matches!(trigger, StatusTriggerType::Spawn))
            })
        {
            let context = EffectContext {
                caster: Some(id),
                from: Some(id),
                target: Some(id),
                vars,
                status_id: Some(status_id),
                color: Some(status_color),
            };
            self.effects.push_front(QueuedEffect {
                effect,
                context: context.clone(),
            });
        }

        self.model.next_id += 1;
        self.model.units.insert(unit);
        id
    }

    pub fn process_spawns(&mut self) {
        for unit in &mut self.model.units {
            if let Some(time) = &mut unit.spawn_animation_time_left {
                *time -= self.delta_time;
                if *time <= Time::new(0.0) {
                    unit.spawn_animation_time_left = None;
                }
            }
        }
    }
}
