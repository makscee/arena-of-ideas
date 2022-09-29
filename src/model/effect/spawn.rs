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
        let mut faction = caster.faction;
        if effect.switch_faction {
            if faction == Faction::Player {
                faction = Faction::Enemy;
            } else {
                faction = Faction::Player;
            }
        }
        let target = context
            .target
            .and_then(|id| {
                logic
                    .model
                    .units
                    .get(&id)
                    .or(logic.model.dead_units.get(&id))
            })
            .expect("Target not found");
        let position = target.position;
        let new_id = logic.spawn_by_type(&effect.unit_type, faction, position);

        logic.effects.push_front(QueuedEffect {
            effect: effect.after_effect.clone(),
            context: {
                let mut context = context.clone();
                context.target = Some(new_id);
                context
            },
        })
    }
}
