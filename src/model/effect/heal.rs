use super::*;

pub type HealType = String;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct HealEffect {
    pub value: Expr,
    #[serde(default)]
    pub types: HashSet<HealType>,
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
            .and_then(|id| logic.model.units.get_mut(&id))
            .expect("Target not found");

        target_unit.stats.max_hp += add_max_hp;
        target_unit.permanent_stats.max_hp += add_max_hp;
        let max_health = target_unit.stats.max_hp + heal_past_max;
        if !effect.no_text {
            let heal_text = (value * r32(10.0)).floor() / r32(10.0);
            logic.model.render_model.add_text(
                target_unit.position,
                &format!("{}", heal_text),
                Color::GREEN,
                crate::render::TextType::Heal,
            );
            target_unit.last_heal_time = logic.model.time;
        }
        let value_clamped = min(value, max_health - target_unit.stats.health);
        target_unit.stats.health += value_clamped;
        target_unit.permanent_stats.health += value_clamped;

        for (effect, mut vars, status_id) in target_unit.all_statuses.iter().flat_map(|status| {
            status.trigger(|trigger| match trigger {
                StatusTrigger::HealTaken { heal_type } => match &heal_type {
                    Some(heal_type) => effect.types.contains(heal_type),
                    None => true,
                },
                _ => false,
            })
        }) {
            logic.effects.push_front(QueuedEffect {
                effect,
                context: EffectContext {
                    caster: context.caster,
                    from: context.from,
                    target: context.target,
                    vars: {
                        vars.extend(context.vars.clone());
                        vars.insert(VarName::HealthRestored, value_clamped);
                        vars.insert(VarName::IncomingHeal, value);
                        vars
                    },
                    status_id: Some(status_id),
                },
            })
        }

        let caster = context
            .caster
            .and_then(|id| {
                logic
                    .model
                    .units
                    .get(&id)
                    .or(logic.model.dead_units.get(&id))
            })
            .expect("Caster not found");
        for (effect, mut vars, status_id) in caster.all_statuses.iter().flat_map(|status| {
            status.trigger(|trigger| match trigger {
                StatusTrigger::HealDealt { heal_type } => match &heal_type {
                    Some(heal_type) => effect.types.contains(heal_type),
                    None => true,
                },
                _ => false,
            })
        }) {
            logic.effects.push_front(QueuedEffect {
                effect,
                context: EffectContext {
                    caster: context.caster,
                    from: context.from,
                    target: context.target,
                    vars: {
                        vars.extend(context.vars.clone());
                        vars.insert(VarName::HealthRestored, value_clamped);
                        vars.insert(VarName::IncomingHeal, value);
                        dbg!(vars)
                    },
                    status_id: Some(status_id),
                },
            })
        }
    }
}
