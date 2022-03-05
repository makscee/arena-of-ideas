use super::*;

pub struct QueuedEffect {
    pub effect: Effect,
    pub caster: Option<Id>,
    pub target: Option<Id>,
}

impl Game {
    pub fn process_effects(&mut self) {
        while let Some(effect) = self.effects.pop() {
            match effect.effect {
                Effect::Damage { hp, kill_effects } => {
                    let target = effect
                        .target
                        .and_then(|id| self.units.get_mut(&id))
                        .expect("Target not found");
                    let mut damage = hp;
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
                        for kill_effect in kill_effects {
                            self.effects.push(QueuedEffect {
                                effect: kill_effect.clone(),
                                caster: effect.caster,
                                target: effect.target,
                            });
                        }
                    }
                }
                Effect::AddStatus { status } => {
                    let target = effect
                        .target
                        .and_then(|id| self.units.get_mut(&id))
                        .expect("Target not found");
                    target.statuses.push(status.clone());
                }
                Effect::Suicide => {
                    if let Some(caster) = effect.caster.and_then(|id| self.units.get_mut(&id)) {
                        caster.hp = Health::new(0.0);
                    }
                }
                Effect::Spawn { unit_type } => {
                    let caster = effect
                        .caster
                        .and_then(|id| self.units.get(&id).or(self.dead_units.get(&id)))
                        .expect("Caster not found");
                    let faction = caster.faction;
                    let target = effect
                        .target
                        .and_then(|id| self.units.get(&id).or(self.dead_units.get(&id)))
                        .expect("Target not found");
                    let position = target.position;
                    self.spawn_unit(&unit_type, faction, position);
                }
                Effect::AOE {
                    radius,
                    filter,
                    effects,
                } => {
                    let caster_faction = match effect.caster.and_then(|id| self.units.get_mut(&id))
                    {
                        Some(caster) => caster.faction,
                        None => todo!(),
                    };
                    let target = effect
                        .target
                        .and_then(|id| self.units.get_mut(&id))
                        .expect("Target not found");
                    let center = target.position;
                    for unit in &self.units {
                        if (unit.position - center).len() - unit.radius() > radius {
                            continue;
                        }
                        match filter {
                            TargetFilter::Allies => {
                                if unit.faction != caster_faction {
                                    continue;
                                }
                            }
                            TargetFilter::Enemies => {
                                if unit.faction == caster_faction {
                                    continue;
                                }
                            }
                            TargetFilter::All => {}
                        }
                        for new_effect in &effects {
                            self.effects.push(QueuedEffect {
                                effect: new_effect.clone(),
                                caster: effect.caster,
                                target: Some(unit.id),
                            });
                        }
                    }
                }
            }
        }
    }
}
