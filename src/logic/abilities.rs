use super::*;

impl Logic<'_> {
    pub fn process_abilities(&mut self) {
        for unit in &mut self.model.units {
            if let Some(time) = &mut unit.ability_cooldown {
                *time -= self.delta_time;
                if *time < Time::new(0.0) {
                    unit.ability_cooldown = None;
                }
            }
        }
        for key in mem::take(&mut self.pressed_keys) {
            if key == "MouseLeft" {
                for unit in &mut self.model.units {
                    let template = &self.model.unit_templates[&unit.unit_type];
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
                        self.effects.push_back(QueuedEffect {
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
            }
            if key == "Space" {
                debug!("space pressed");
            }
        }
    }
}
