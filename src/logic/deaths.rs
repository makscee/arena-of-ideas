use super::*;

impl Logic {
    pub fn kill(&mut self, id: Id) {
        let unit = self.model.units.get_mut(&id).unwrap();
        unit.permanent_stats.health = 0;
        let unit = self.model.units.get(&id).unwrap();

        for other in self.model.units.iter().filter(|other| other.id != unit.id) {
            for (effect, trigger, vars, status_id, status_color) in
                other.all_statuses.iter().flat_map(|status| {
                    status.trigger(|trigger| match trigger {
                        StatusTriggerType::Scavenge { who, range, clan } => {
                            who.matches(other.faction, unit.faction)
                                && clan.map(|clan| unit.clans.contains(&clan)).unwrap_or(true)
                                && distance_between_units(other, unit) > *range
                        }
                        _ => false,
                    })
                })
            {
                self.effects.push_back(QueuedEffect {
                    effect,
                    context: EffectContext {
                        caster: Some(other.id),
                        from: Some(other.id),
                        target: Some(unit.id),
                        vars,
                        status_id: Some(status_id),
                        color: Some(status_color),
                    },
                })
            }
        }
    }
    pub fn process_deaths(&mut self) {
        if self.model.visual_timer > Time::ZERO {
            return;
        }
        let ids = self.model.units.ids().copied().collect::<Vec<_>>();
        for id in ids {
            let unit = self.model.units.get(&id).unwrap();
            if unit.permanent_stats.health <= 0 {
                self.model.dead_units.insert(unit.clone());
                for (effect, trigger, vars, status_id, status_color) in
                    unit.all_statuses.iter().flat_map(|status| {
                        status.trigger(|trigger| matches!(trigger, StatusTriggerType::Death))
                    })
                {
                    let context = EffectContext {
                        caster: Some(unit.id),
                        from: Some(unit.id),
                        target: Some(unit.id),
                        vars,
                        status_id: Some(status_id),
                        color: Some(status_color),
                    };
                    trigger.fire(effect, &context, &mut self.effects);
                }
                let unit_position = unit.clone().position;
                self.update_positions(id, unit_position);
            }
        }

        self.model.units.retain(|unit| unit.permanent_stats.health > 0);
    }
    fn update_positions(&mut self, unit_id: Id, unit_position: Position) {
        // Move horizontally
        self.model
            .units
            .iter_mut()
            .filter(|other| {
                other.position.side == unit_position.side && other.position.x > unit_position.x
            })
            .for_each(|other| other.position.x -= 1);
    }
}
