use super::*;

impl<'a> Logic {
    pub fn init_abilities(&mut self, events: &mut Events) {
        events.add_listener(
            GameEvent::Ability,
            Box::new(|logic| {
                for mut unit in &mut logic.model.units {
                    let template = &logic.model.unit_templates[&unit.unit_type];
                    if unit.ability_cooldown.is_some() {
                        continue;
                    }
                    if unit.faction != Faction::Player {
                        continue;
                    }
                    if unit
                        .flags
                        .iter()
                        .any(|flag| matches!(flag, UnitStatFlag::ActionUnable))
                    {
                        continue;
                    }
                    if let Some(ability) = &template.ability {
                        unit.ability_cooldown = Some(ability.cooldown);
                        logic.effects.push_back(QueuedEffect {
                            effect: ability.effect.clone(),
                            context: EffectContext {
                                caster: Some(unit.id),
                                from: Some(unit.id),
                                target: Some(unit.id),
                                vars: default(),
                                status_id: None,
                            },
                        });
                    }
                }
            }),
        );
    }

    pub fn process_abilities(&mut self) {
        for unit in &mut self.model.units {
            if let Some(time) = &mut unit.ability_cooldown {
                *time -= self.delta_time;
                if *time < Time::new(0.0) {
                    unit.ability_cooldown = None;
                }
            }
        }
    }
}
