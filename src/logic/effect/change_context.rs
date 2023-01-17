use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChangeContextEffect {
    #[serde(default)]
    pub owner: Option<Who>,
    #[serde(default)]
    pub creator: Option<Who>,
    #[serde(default)]
    pub target: Option<Who>,
    pub color: Option<Rgba<f32>>,
    pub effect: LogicEffect,
    #[serde(default)]
    pub default_vars: HashMap<VarName, i32>,
}

impl EffectContainer for ChangeContextEffect {
    fn walk_effects_mut(&mut self, f: &mut dyn FnMut(&mut LogicEffect)) {
        self.effect.walk_mut(f);
    }
}

impl EffectImpl for ChangeContextEffect {
    fn process(self: Box<Self>, context: LogicEffectContext, logic: &mut logic::Logic) {
        let effect = *self;
        let mut vars = context.vars.clone();
        for entry in &effect.default_vars {
            if !vars.contains_key(&entry.0) {
                vars.insert(entry.0.clone(), entry.1.clone());
            }
        }
        // todo: use new queue
        // logic.effects.push_front(
        //     LogicEffectContext {
        //         creator: match effect.creator {
        //             Some(who) => context.get_id(who),
        //             None => context.creator,
        //         },
        //         owner: match effect.owner {
        //             Some(who) => context.get_id(who),
        //             None => context.owner,
        //         },
        //         target: match effect.target {
        //             Some(who) => context.get_id(who),
        //             None => context.target,
        //         },
        //         color: match effect.color {
        //             Some(color) => color,
        //             None => context.color,
        //         },
        //         vars,
        //         ..context
        //     },
        //     effect.effect,
        // );
    }
}
