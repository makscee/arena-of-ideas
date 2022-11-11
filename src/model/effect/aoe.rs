use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AoeEffect {
    pub filter: TargetFilter,
    #[serde(default)]
    pub skip_current_target: bool,
    pub range: Option<Coord>,
    pub effect: Effect,
}

impl EffectContainer for AoeEffect {
    fn walk_effects_mut(&mut self, f: &mut dyn FnMut(&mut Effect)) {
        self.effect.walk_mut(f);
    }
}

impl EffectImpl for AoeEffect {
    fn process(self: Box<Self>, context: EffectContext, logic: &mut logic::Logic) {
        let effect = *self;
        let owner = logic.model.get(Who::Owner, &context);
        let owner_faction = owner.faction;
        let center = logic.model.get(Who::Owner, &context).position;
        logic
            .model
            .render_model
            .add_text(center, "AOE", Rgba::RED, crate::render::TextType::Aoe);
        for unit in &logic.model.units {
            if effect.skip_current_target && unit.id == context.target {
                continue;
            }
            if let Some(range) = effect.range {
                if unit.position.distance(&center) > range {
                    continue;
                }
            }
            if !effect.filter.matches(unit.faction, owner_faction) {
                continue;
            }
            logic.effects.push_front(
                {
                    let mut context = context.clone();
                    context.target = unit.id;
                    context
                },
                effect.effect.clone(),
            );
        }
    }
}
