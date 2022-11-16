use super::*;

impl Logic {
    /// Spawns the unit and returns its id. If there is a unit in that position and there is an
    /// empty slot to the left, it and all units to the left are shifted to the left
    pub fn spawn_by_type(&mut self, unit_type: &UnitType, position: Position) -> Id {
        let mut unit = self.create_by_type(unit_type, position);
        self.spawn_by_unit(unit)
    }

    /// Create unit without putting it in model.units
    pub fn create_by_type(&mut self, unit_type: &UnitType, position: Position) -> Unit {
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
        self.model.next_id += 1;
        unit
    }

    pub fn apply_spawn_effects(&mut self, unit: &mut Unit) {
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
                unit,
                &self.model.clan_effects,
                size,
                self.model.next_id,
                &self.model.statuses,
            );
        }

        // On spawn effects
        for (effect, trigger, vars, status_id, status_color) in unit
            .all_statuses
            .iter()
            .flat_map(|status| status.trigger(|trigger| matches!(trigger, StatusTrigger::Spawn)))
        {
            let context = EffectContext {
                owner: unit.id,
                creator: unit.id,
                target: unit.id,
                vars,
                status_id: Some(status_id),
                color: status_color,
                queue_id: Some("Spawn".to_owned()),
            };
            self.effects.push_back(context, effect);
        }
    }

    pub fn spawn_by_unit(&mut self, mut unit: Unit) -> Id {
        let id = self.model.next_id;
        self.model.next_id += 1;
        unit.id = id;
        debug!("Spawn by unit {}#{}", unit.unit_type, unit.id);
        let position = unit.position;
        // Check empty slots
        // Shift the units, assuming that there are no empty slots in between
        self.model
            .units
            .iter_mut()
            .filter(|unit| unit.position.side == position.side && unit.position.x >= position.x)
            .for_each(|unit| unit.position.x += 1);
        self.apply_spawn_effects(&mut unit);
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
