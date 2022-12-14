use super::*;

impl Logic {
    /// Spawns the unit and returns its id. If there is a unit in that position and there is an
    /// empty slot to the left, it and all units to the left are shifted to the left
    pub fn spawn_by_type(&mut self, unit_type: &UnitType, position: Position) -> Id {
        let mut unit = self.create_by_type(unit_type, position);
        self.spawn_by_unit(unit, false)
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

    pub fn apply_clan_effects(&mut self, unit: &mut Unit) {
        for (clan, config) in &self.model.clans.map {
            config.effects.iter().for_each(|effect| {
                let context = EffectContext {
                    owner: unit.id,
                    creator: unit.id,
                    target: unit.id,
                    vars: self.model.vars.clone(),
                    status_id: None,
                    color: config.color,
                    queue_id: Some("Spawn".to_owned()),
                };
                self.effects.push_back(context.clone(), effect.clone());
            });
        }
    }

    pub fn apply_spawn_effects(&mut self, unit: &mut Unit) {
        for trigger_effect in unit
            .trigger()
            .filter(|effect| matches!(effect.trigger, StatusTrigger::Spawn))
        {
            let context = EffectContext {
                owner: unit.id,
                creator: unit.id,
                target: unit.id,
                vars: trigger_effect.vars,
                status_id: Some(trigger_effect.status_id),
                color: trigger_effect.status_color,
                queue_id: Some("Spawn".to_owned()),
            };
            self.effects.push_back(context, trigger_effect.effect);
        }
    }

    pub fn spawn_by_unit(&mut self, mut unit: Unit, no_shift: bool) -> Id {
        let id = self.model.next_id;
        self.model.next_id += 1;
        unit.id = id;
        debug!("Spawn by unit {}#{}", unit.unit_type, unit.id);
        let position = unit.position;
        // Check empty slots
        // Shift the units, assuming that there are no empty slots in between
        if !no_shift {
            self.model
                .units
                .iter_mut()
                .filter(|unit| unit.position.side == position.side && unit.position.x >= position.x)
                .for_each(|unit| unit.position.x += 1);
        }
        self.apply_spawn_effects(&mut unit);
        self.apply_clan_effects(&mut unit);
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
