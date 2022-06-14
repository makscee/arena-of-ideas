use super::*;

fn default_who() -> Who {
    Who::Target
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RemoveStatusEffect {
    #[serde(default = "default_who")]
    pub who: Who,
    pub status: StatusName,
    #[serde(default)]
    pub all: bool,
}

impl EffectContainer for RemoveStatusEffect {
    fn walk_effects_mut(&mut self, _f: &mut dyn FnMut(&mut Effect)) {}
}

impl EffectImpl for RemoveStatusEffect {
    fn process(self: Box<Self>, context: EffectContext, logic: &mut logic::Logic) {
        let effect = *self;
        let status_name = &effect.status;
        let target = context.get(effect.who);
        let all = effect.all;
        if let Some(target) = target {
            let target = logic.model.units.get_mut(&target).unwrap();
            for status in &mut target.all_statuses {
                if status.status.name == *status_name {
                    status.time = Some(Time::ZERO);
                    if !all {
                        return;
                    }
                }
            }
        }
    }
}
