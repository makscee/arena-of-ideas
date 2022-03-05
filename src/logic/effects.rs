use super::*;

impl Game {
    pub fn apply_effect(&mut self, effect: &Effect, caster: Option<&mut Unit>, target: &mut Unit) {
        match effect {
            Effect::AddStatus { status } => {
                target.statuses.push(status.clone());
            }
            Effect::Suicide => {
                if let Some(caster) = caster {
                    caster.hp = Health::new(-100500.0);
                }
            }
            Effect::Spawn { unit_type } => self.spawn_unit(
                unit_type,
                match caster {
                    Some(unit) => unit.faction,
                    None => target.faction,
                },
                target.position,
            ),
            Effect::AOE {
                radius,
                filter,
                effects,
            } => {
                let mut caster = caster;
                let center = target.position;
                let caster_faction = match &caster {
                    Some(caster) => caster.faction,
                    None => todo!(),
                };
                self.process_units(|this, unit| {
                    if (unit.position - center).len() - unit.radius() > *radius {
                        return;
                    }
                    match filter {
                        TargetFilter::Allies => {
                            if unit.faction != caster_faction {
                                return;
                            }
                        }
                        TargetFilter::Enemies => {
                            if unit.faction == caster_faction {
                                return;
                            }
                        }
                        TargetFilter::All => {}
                    }
                    for effect in effects {
                        this.apply_effect(effect, caster.as_deref_mut(), unit);
                    }
                });
            }
        }
    }
}
