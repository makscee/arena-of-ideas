use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SpawnEffect {
    pub unit_type: UnitType,
    #[serde(default)]
    pub switch_faction: bool,
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
        let new_id = logic.spawn_unit(&effect.unit_type, faction, position);
        if context.vars.contains_key(&VarName::SpawnHealth) {
            logic.effects.push_back(QueuedEffect {
                effect: Effect::List(Box::new(ListEffect {
                    effects: vec![
                        Effect::ChangeStat(Box::new(ChangeStatEffect {
                            stat: UnitStat::MaxHealth,
                            value: Expr::Var {
                                name: VarName::SpawnHealth,
                            },
                        })),
                        Effect::Heal(Box::new(HealEffect {
                            value: Expr::Var {
                                name: VarName::SpawnHealth,
                            },
                            heal_past_max: default(),
                            add_max_hp: default(),
                        })),
                    ],
                })),
                context: EffectContext {
                    target: Some(new_id),
                    ..context.clone()
                },
            });
        }
    }
}
