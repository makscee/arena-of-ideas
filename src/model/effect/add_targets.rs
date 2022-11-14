use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct AddTargetsEffect {
    pub additional_targets: Option<usize>,
    #[serde(default)]
    pub condition: Condition,
    pub effect: Effect,
    #[serde(default)]
    pub replace: bool, // remove original Target
}

impl EffectContainer for AddTargetsEffect {
    fn walk_effects_mut(&mut self, f: &mut dyn FnMut(&mut Effect)) {
        self.effect.walk_mut(f);
    }
}

impl EffectImpl for AddTargetsEffect {
    fn process(self: Box<Self>, context: EffectContext, logic: &mut logic::Logic) {
        let effect = *self;
        let owner = logic.model.get_who(Who::Owner, &context);
        let target = logic.model.get_who(Who::Target, &context);
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
                .filter(|unit| {
                    Logic::check_condition(
                        &logic.model,
                        &effect.condition,
                        &EffectContext {
                            target: unit.id,
                            ..context.clone()
                        },
                    )
                })
                .choose(&mut global_rng())
            {
                targets.insert(another.id);
            } else {
                break;
            }
        }
        if effect.replace {
            targets.remove(&target.id);
        }
        for target in targets {
            logic.effects.push_front(
                {
                    let mut context = context.clone();
                    context.target = target;
                    context
                },
                effect.effect.clone(),
            );
        }
    }
}
