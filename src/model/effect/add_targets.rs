use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct AddTargetsEffect {
    pub additional_targets: Option<usize>,
    #[serde(default)]
    pub condition: Condition,
    pub effect: Effect,
}

impl EffectContainer for AddTargetsEffect {
    fn walk_effects_mut(&mut self, f: &mut dyn FnMut(&mut Effect)) {
        self.effect.walk_mut(f);
    }
}

impl EffectImpl for AddTargetsEffect {
    fn process(self: Box<Self>, context: EffectContext, logic: &mut logic::Logic) {
        let effect = *self;
        let from = context
            .from
            .and_then(|id| logic.model.units.get(&id))
            .expect("From not found");
        let target = context
            .target
            .and_then(|id| logic.model.units.get(&id))
            .expect("Target not found");
        let mut targets: HashSet<Id> = default();
        targets.insert(target.id);
        while match effect.additional_targets {
            Some(num) => targets.len() < 1 + num,
            None => true,
        } {
            if let Some(another) = logic
                .model
                .units
                .iter()
                .filter(|unit| unit.faction == target.faction)
                .filter(|unit| !targets.contains(&unit.id))
                .filter(|unit| distance_between_units(unit, from) < from.range)
                .filter(|unit| {
                    logic.check_condition(&effect.condition, &EffectContext { ..context.clone() })
                })
                .choose(&mut global_rng())
            {
                targets.insert(another.id);
            } else {
                break;
            }
        }
        for target in targets {
            logic.effects.push_front(QueuedEffect {
                effect: effect.effect.clone(),
                context: EffectContext {
                    target: Some(target),
                    ..context.clone()
                },
            });
        }
    }
}
