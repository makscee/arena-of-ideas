use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SpawnEffect {
    pub unit_type: UnitType,
    #[serde(default)]
    pub switch_faction: bool,
    #[serde(default)]
    pub after_effect: Effect,
}

impl EffectContainer for SpawnEffect {
    fn walk_effects_mut(&mut self, _f: &mut dyn FnMut(&mut Effect)) {}
}

impl EffectImpl for SpawnEffect {
    fn process(self: Box<Self>, context: EffectContext, logic: &mut logic::Logic) {
        let effect = *self;
        let owner = logic.model.get(Who::Owner, &context);
        let mut faction = owner.faction;
        if effect.switch_faction {
            if faction == Faction::Player {
                faction = Faction::Enemy;
            } else {
                faction = Faction::Player;
            }
        }
        let target = logic.model.get(Who::Target, &context);
        let mut position = target.position;
        position.side = faction;
        let new_id = logic.spawn_by_type(&effect.unit_type, position);

        logic.effects.push_front(
            {
                let mut context = context.clone();
                context.target = new_id;
                context
            },
            effect.after_effect.clone(),
        )
    }
}
