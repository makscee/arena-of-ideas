use super::*;

fn default_who() -> Who {
    Who::Target
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChangeContextStatusEffect {
    #[serde(default = "default_who")]
    pub who: Who,
    pub status: StatusName,
    pub effect: Effect,
}

impl EffectContainer for ChangeContextStatusEffect {
    fn walk_effects_mut(&mut self, f: &mut dyn FnMut(&mut Effect)) {
        self.effect.walk_mut(f);
    }
}

impl EffectImpl for ChangeContextStatusEffect {
    fn process(self: Box<Self>, context: EffectContext, logic: &mut logic::Logic) {
        let effect = *self;
        let target = context
            .target
            .and_then(|id| {
                logic
                    .model
                    .units
                    .get(&id)
                    .or_else(|| logic.model.units.get(&id))
            })
            .expect("Target not found");
        let status = target
            .all_statuses
            .iter()
            .find(|status| status.status.name == effect.status);
        if let Some(status) = status {
            logic.effects.push_front(QueuedEffect {
                effect: effect.effect,
                context: EffectContext {
                    status_id: Some(status.id),
                    ..context
                },
            });
        } else {
            warn!("Status not found {:?}", effect.status);
        }
    }
}
