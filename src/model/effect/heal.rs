use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct HealEffect {
    pub value: Expr,
    #[serde(default)]
    pub heal_past_max: Option<Expr>,
    #[serde(default)]
    pub add_max_hp: Option<Expr>,
    #[serde(default)]
    pub no_text: bool,
}

impl EffectContainer for HealEffect {
    fn walk_effects_mut(&mut self, _f: &mut dyn FnMut(&mut Effect)) {}
}

impl EffectImpl for HealEffect {
    fn process(self: Box<Self>, context: EffectContext, logic: &mut logic::Logic) {
        let effect = *self;

        let add_max_hp = match &effect.add_max_hp {
            Some(expr) => expr.calculate(&context, logic),
            None => R32::ZERO,
        };
        let heal_past_max = match &effect.heal_past_max {
            Some(expr) => expr.calculate(&context, logic),
            None => R32::ZERO,
        };
        let value = effect.value.calculate(&context, logic);

        let target_unit = context
            .target
            .and_then(|id| {
                logic
                    .model
                    .units
                    .get_mut(&id)
                    .or(logic.model.spawning_units.get_mut(&id))
            })
            .expect("Target not found");

        target_unit.max_hp += add_max_hp;
        let max_health = target_unit.max_hp + heal_past_max;
        if !effect.no_text {
            if let Some(render) = &mut logic.render {
                let heal_text = (value * r32(10.0)).floor() / r32(10.0);
                render.add_text(
                    target_unit.position,
                    &format!("{}", heal_text),
                    Color::GREEN,
                );
            }
        }
        let value = min(value, max_health - target_unit.health);
        target_unit.health += value;

        for status in &target_unit.all_statuses {
            // TODO: reimplement
            // if let StatusOld::OnHeal(status) = status {
            //     logic.effects.push_front(QueuedEffect {
            //         effect: status.effect.clone(),
            //         context: {
            //             let mut context = context.clone();
            //             context.vars.insert(VarName::HealthRestored, value);
            //             context
            //         },
            //     });
            // }
        }
    }
}
