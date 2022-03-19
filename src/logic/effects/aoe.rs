use super::*;

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
            if effect.skip_current_target && Some(unit.id) == context.target {
                continue;
            }
            if (unit.position - center).len() - unit.radius() > effect.radius {
                continue;
            }
            if !effect.filter.matches(unit.faction, caster_faction) {
                continue;
            }
            self.effects.push_back(QueuedEffect {
                effect: effect.effect.clone(),
                context: EffectContext {
                    caster: context.caster,
                    from: context.target,
                    target: Some(unit.id),
                },
            });
        }
    }
}
