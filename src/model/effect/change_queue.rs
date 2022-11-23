use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChangeQueueIdEffect {
    pub queue_id: String,
    #[serde(default)]
    pub owner_postfix: bool,
    pub effect: Effect,
}

impl EffectContainer for ChangeQueueIdEffect {
    fn walk_effects_mut(&mut self, f: &mut dyn FnMut(&mut Effect)) {
        self.effect.walk_mut(f);
    }
}

impl EffectImpl for ChangeQueueIdEffect {
    fn process(self: Box<Self>, context: EffectContext, logic: &mut logic::Logic) {
        let effect = *self;
        let postfix = if effect.owner_postfix {
            context.owner.to_string()
        } else {
            "".to_string()
        };
        let queue_id = format!("{}{}", effect.queue_id, postfix);
        debug!("q_id: {}", queue_id);
        logic.effects.push_front(
            EffectContext {
                queue_id: Some(queue_id),
                ..context
            },
            effect.effect,
        );
    }
}
