use super::*;

fn default_who() -> Who {
    Who::Target
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChangeStatEffect {
    pub stat: UnitStat,
    pub value: Expr,
    #[serde(default = "default_who")]
    pub who: Who,
}

impl EffectContainer for ChangeStatEffect {
    fn walk_effects_mut(&mut self, f: &mut dyn FnMut(&mut Effect)) {}
}

impl EffectImpl for ChangeStatEffect {
    fn process(self: Box<Self>, context: EffectContext, logic: &mut Logic) {
        let effect = *self;
        let value = effect.value.calculate(&context, logic);
        let target = context.get(effect.who).unwrap();
        let target = logic
            .model
            .units
            .get_mut(&target)
            .expect("Target not found");
        *target.stats.get_mut(effect.stat) = value;
        *target.permanent_stats.get_mut(effect.stat) = value;
    }
}
