use super::*;

fn default_who() -> Who {
    Who::Target
}

fn default_permanent() -> bool {
    false
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChangeStatEffect {
    pub stat: UnitStat,
    pub value: Expr,
    #[serde(default = "default_who")]
    pub who: Who,
    #[serde(default = "default_permanent")]
    pub permanent: bool,
}

impl EffectContainer for ChangeStatEffect {
    fn walk_effects_mut(&mut self, f: &mut dyn FnMut(&mut Effect)) {}
}

impl EffectImpl for ChangeStatEffect {
    fn process(self: Box<Self>, context: EffectContext, logic: &mut Logic) {
        let effect = *self;
        let value = effect.value.calculate(&context, &logic.model);
        let mut target = logic.model.get_mut(effect.who, &context);
        let change_value = value - target.stats.get_mut(effect.stat).clone();
        *target.stats.get_mut(effect.stat) = value;
        *target.permanent_stats.get_mut(effect.stat) = value;

        if effect.permanent && target.shop_unit.is_some() {
            let mut shop_unit = target.shop_unit.clone().unwrap();
            *shop_unit.stats.get_mut(effect.stat) += change_value;
            *shop_unit.permanent_stats.get_mut(effect.stat) += change_value;
            target.shop_unit = Box::new(Some(shop_unit));
        }
    }
}
