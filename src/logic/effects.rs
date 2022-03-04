use super::*;

impl Game {
    pub fn apply_effect(
        &mut self,
        effect: &Effect,
        mut caster: Option<&mut Unit>,
        target: &mut Unit,
    ) {
        match effect {
            Effect::AddStatus { status } => {
                target.statuses.push(status.clone());
            }
            Effect::Suicide => {
                if let Some(caster) = &mut caster {
                    caster.hp = -100500;
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
                    target.position
                        + vec2(
                            global_rng().gen_range(Coord::new(-1.0)..=Coord::new(1.0)),
                            global_rng().gen_range(Coord::new(-1.0)..=Coord::new(1.0)),
                        ) * Coord::new(0.01),
                )
            }
        }
    }
}
