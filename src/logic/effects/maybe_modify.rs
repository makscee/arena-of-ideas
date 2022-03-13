use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", deny_unknown_fields)]
pub struct MaybeModifyEffect {
    pub base_effect: Effect,
    pub condition: Condition,
    pub modifier: Modifier,
}

impl MaybeModifyEffect {
    pub fn walk_children_mut(&mut self, f: &mut impl FnMut(&mut Effect)) {
        self.base_effect.walk_mut(f);
    }
}

impl Logic<'_> {
    pub fn process_maybe_modify_effect(
        &mut self,
        QueuedEffect {
            mut effect,
            caster,
            target,
        }: QueuedEffect<MaybeModifyEffect>,
    ) {
        let condition = match &effect.condition {
            Condition::UnitHasStatus { who, status } => {
                let who = match who {
                    Who::Caster => caster,
                    Who::Target => target,
                };
                let who = who
                    .and_then(|id| self.model.units.get(&id))
                    .expect("Caster or Target not found");
                who.all_statuses.contains(status)
            }
        };

        if condition {
            effect.base_effect.apply_modifier(&effect.modifier);
        }
        self.effects.push_back(QueuedEffect {
            effect: effect.base_effect,
            caster,
            target,
        })
    }
}
