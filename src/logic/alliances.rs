use super::*;

impl Alliance {
    pub fn apply(&self, template: &mut UnitTemplate) {
        match self {
            Self::Assassins => {
                let crit_percent = 15.0;
                template
                    .attack
                    .effect
                    .walk_mut(&mut |effect| match &effect {
                        Effect::Damage(damage) => {
                            *effect = Effect::Random {
                                choices: vec![
                                    WeighedEffect {
                                        weight: 100.0 - crit_percent,
                                        effect: effect.clone(),
                                    },
                                    WeighedEffect {
                                        weight: crit_percent,
                                        effect: Effect::List {
                                            effects: vec![
                                                Effect::Damage(Box::new(DamageEffect {
                                                    hp: damage.hp * r32(3.0),
                                                    lifesteal: damage.lifesteal,
                                                    types: {
                                                        let mut types = damage.types.clone();
                                                        types.insert("Crit".to_owned());
                                                        types
                                                    },
                                                    on: damage.on.clone(),
                                                })),
                                                Effect::AddStatus(Box::new(AddStatusEffect {
                                                    who: Who::Target,
                                                    status: Status::Slow {
                                                        percent: 70.0,
                                                        time: r32(3.0),
                                                    },
                                                })),
                                                Effect::AddStatus(Box::new(AddStatusEffect {
                                                    who: Who::Caster,
                                                    status: Status::Shield,
                                                })),
                                            ],
                                        },
                                    },
                                ],
                            };
                        }
                        _ => {}
                    });
            }
            Self::Spawners => {}
        }
    }
}
