use super::*;

impl Logic {
    /// Spawns the unit and returns its id. If there is a unit in that position and there is an
    /// empty slot to the left, it and all units to the left are shifted to the left.
    /// Otherwise, if all slots are occupied, the unit is placed on top the unit in that position.
    pub fn spawn_unit(&mut self, unit_type: &UnitType, faction: Faction, position: Position) -> Id {
        let mut template = &self
            .model
            .unit_templates
            .get(unit_type)
            .unwrap_or_else(|| panic!("Failed to find unit template for {unit_type}"));
        let id = self.model.next_id;

        // Check empty slots
        let can_shift = SIDE_SLOTS
            .checked_sub(1)
            .map(|max_pos| {
                let max_pos = max_pos as Coord;
                self.model
                    .units
                    .iter()
                    .filter(|unit| unit.position.side == position.side)
                    .all(|unit| unit.position.x < max_pos)
            })
            .expect("Expected at least one slot for the team");
        let height = if can_shift {
            // Shift the units, assuming that there are no empty slots in between
            self.model
                .units
                .iter_mut()
                .filter(|unit| unit.position.side == position.side && unit.position.x >= position.x)
                .for_each(|unit| unit.position.x += 1);
            0
        } else {
            self.model
                .units
                .iter()
                .filter(|unit| unit.position.side == position.side && unit.position.x == position.x)
                .map(|unit| unit.position.height)
                .max()
                .map(|y| y + 1)
                .unwrap_or(0)
        };
        let position = Position { height, ..position };

        let mut unit = Unit::new(
            &template,
            &mut self.model.next_id,
            unit_type.clone(),
            faction,
            position,
            &self.model.statuses,
        );
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
                &mut self.model.next_id,
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
            trigger.fire(effect, &context, &mut self.effects);
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
