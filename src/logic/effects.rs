use super::*;

impl Game {
    pub fn apply_effect(
        &mut self,
        effect: &Effect,
        mut caster: Option<&mut Unit>,
        target: &mut Unit,
    ) {
        match effect {
            Effect::Damage { hp, kill_effects } => {
                let mut damage = *hp;
                damage = min(damage, target.hp);
                if damage > Health::new(0.0) {
                    if let Some((index, _)) = target
                        .statuses
                        .iter()
                        .enumerate()
                        .find(|(_, status)| matches!(status, Status::Shield))
                    {
                        damage = Health::new(0.0);
                        target.statuses.remove(index);
                    }
                }
                if damage > Health::new(0.0) {
                    target
                        .statuses
                        .retain(|status| !matches!(status, Status::Freeze));
                }
                let old_hp = target.hp;
                target.hp -= damage;
                if old_hp > Health::new(0.0) && target.hp <= Health::new(0.0) {
                    for effect in kill_effects {
                        self.apply_effect(effect, caster.as_deref_mut(), target);
                    }
                }
            }
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
