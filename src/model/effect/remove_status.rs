use super::*;

fn default_who() -> Who {
    Who::Target
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RemoveStatusEffect {
    #[serde(default = "default_who")]
    pub who: Who,
    pub status: Option<StatusName>,
    #[serde(default)]
    pub all: bool, // Remove all statuses with that name
}

impl EffectContainer for RemoveStatusEffect {
    fn walk_effects_mut(&mut self, _f: &mut dyn FnMut(&mut Effect)) {}
}

impl EffectImpl for RemoveStatusEffect {
    fn process(self: Box<Self>, context: EffectContext, logic: &mut logic::Logic) {
        let effect = *self;
        let status_name = &effect.status;
        let status_id = context.status_id;
        let target = logic.model.get_who_mut(Who::Target, &context);
        let all = effect.all;
        for status in &mut target.all_statuses {
            if match status_name {
                Some(name) => *name == status.status.name,
                None => match status_id {
                    Some(id) => id == status.id,
                    None => false,
                },
            } {
                status.time = Some(0);
                debug!("Remove status {}#{}", status.status.name, status.id);
                if !all {
                    return;
                }
            }
        }
    }
}
