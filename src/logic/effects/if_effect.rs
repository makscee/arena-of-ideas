use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", deny_unknown_fields)]
pub struct IfEffect {
    pub condition: Condition,
    pub then: Effect,
    pub r#else: Effect,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum Condition {
    UnitHasStatus { who: Who, status: Status },
}

impl IfEffect {
    pub fn walk_children_mut(&mut self, f: &mut impl FnMut(&mut Effect)) {
        self.then.walk_mut(f);
        self.r#else.walk_mut(f);
    }
}

impl Logic<'_> {
    pub fn process_if_effect(&mut self, QueuedEffect { effect, context }: QueuedEffect<IfEffect>) {
        let condition = match &effect.condition {
            Condition::UnitHasStatus { who, status } => {
                let who = context.get(*who);
                let who = who
                    .and_then(|id| self.model.units.get(&id))
                    .expect("Caster or Target not found");
                who.all_statuses.contains(status)
            }
        };

        let effect = if condition {
            effect.then
        } else {
            effect.r#else
        };
        self.effects.push_back(QueuedEffect { effect, context })
    }
}
