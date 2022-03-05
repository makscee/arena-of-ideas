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
            Effect::Spawn { unit_type } => {
                let template = self.assets.units[unit_type].clone();
                self.spawn_unit(
                    &template,
                    match caster {
                        Some(unit) => unit.faction,
                        None => target.faction,
                    },
                    target.position,
                )
            }
        }
    }
}
