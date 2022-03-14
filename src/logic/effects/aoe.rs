use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AoeEffect {
    pub filter: TargetFilter,
    pub radius: Coord,
    pub effect: Effect,
}

impl AoeEffect {
    pub fn walk_children_mut(&mut self, f: &mut impl FnMut(&mut Effect)) {
        self.effect.walk_mut(f);
    }
}

impl Logic<'_> {
    pub fn process_aoe_effect(
        &mut self,
        QueuedEffect { effect, context }: QueuedEffect<AoeEffect>,
    ) {
        let caster = context
            .caster
            .and_then(|id| self.model.units.get(&id).or(self.model.dead_units.get(&id)))
            .expect("Caster not found");
        let caster_faction = caster.faction;
        let center = context
            .target
            .and_then(|id| {
                self.model.units.get(&id).map(|unit| unit.position).or(self
                    .model
                    .dead_time_bombs
                    .get(&id)
                    .map(|bomb| bomb.position))
            })
            .expect("Target not found");
        if let Some(render) = &mut self.render {
            render.add_text(center, "AOE", Color::RED);
        }
        for unit in &self.model.units {
            if (unit.position - center).len() - unit.radius() > effect.radius {
                continue;
            }
            match effect.filter {
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
            self.effects.push_back(QueuedEffect {
                effect: effect.effect.clone(),
                context: EffectContext {
                    target: Some(unit.id),
                    ..context
                },
            });
        }
    }
}
