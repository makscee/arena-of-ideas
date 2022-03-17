pub use super::*;

impl Logic<'_> {
    pub fn process_add_targets_effect(
        &mut self,
        QueuedEffect { effect, context }: QueuedEffect<AddTargetsEffect>,
    ) {
        let caster = context
            .caster
            .and_then(|id| self.model.units.get(&id))
            .expect("Caster not found");
        let target = context
            .target
            .and_then(|id| self.model.units.get(&id))
            .expect("Target not found");
        let mut targets: HashSet<Id> = default();
        targets.insert(target.id);
        while match effect.additional_targets {
            Some(num) => targets.len() < 1 + num,
            None => true,
        } {
            if let Some(another) = self
                .model
                .units
                .iter()
                .filter(|unit| unit.faction == target.faction)
                .filter(|unit| !targets.contains(&unit.id))
                .filter(|unit| distance_between_units(unit, caster) < caster.action.radius)
                .filter(|unit| {
                    self.check_condition(&effect.condition, &EffectContext { ..context })
                })
                .choose(&mut global_rng())
            {
                targets.insert(another.id);
            } else {
                break;
            }
        }
        for target in targets {
            self.effects.push_back(QueuedEffect {
                effect: effect.effect.clone(),
                context: EffectContext {
                    caster: Some(caster.id),
                    target: Some(target),
                    ..context
                },
            });
        }
    }
}
