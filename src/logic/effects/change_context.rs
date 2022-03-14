use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChangeContextEffect {
    #[serde(default)]
    pub caster: Option<Who>,
    #[serde(default)]
    pub from: Option<Who>,
    #[serde(default)]
    pub target: Option<Who>,
    pub effect: Effect,
}

impl ChangeContextEffect {
    pub fn walk_children_mut(&mut self, f: &mut impl FnMut(&mut Effect)) {
        self.effect.walk_mut(f);
    }
}

impl Logic<'_> {
    pub fn process_change_context_effect(
        &mut self,
        QueuedEffect { effect, context }: QueuedEffect<ChangeContextEffect>,
    ) {
        self.effects.push_back(QueuedEffect {
            effect: effect.effect,
            context: EffectContext {
                caster: match effect.caster {
                    Some(who) => context.get(who),
                    None => context.caster,
                },
                from: match effect.from {
                    Some(who) => context.get(who),
                    None => context.from,
                },
                target: match effect.target {
                    Some(who) => context.get(who),
                    None => context.target,
                },
            },
        });
    }
}
