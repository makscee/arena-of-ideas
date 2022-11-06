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
    pub color: Option<Rgba<f32>>,
    pub effect: Effect,
    #[serde(default)]
    pub default_vars: HashMap<VarName, i32>,
}

impl EffectContainer for ChangeContextEffect {
    fn walk_effects_mut(&mut self, f: &mut dyn FnMut(&mut Effect)) {
        self.effect.walk_mut(f);
    }
}

impl EffectImpl for ChangeContextEffect {
    fn process(self: Box<Self>, context: EffectContext, logic: &mut logic::Logic) {
        let effect = *self;
        let mut vars = context.vars.clone();
        for entry in &effect.default_vars {
            if !vars.contains_key(&entry.0) {
                vars.insert(entry.0.clone(), entry.1.clone());
            }
        }
        logic.effects.push_front(QueuedEffect {
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
                color: match effect.color {
                    Some(color) => Some(color),
                    None => context.color,
                },
                vars,
                ..context
            },
        });
    }
}
