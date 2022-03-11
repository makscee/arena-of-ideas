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
            Self::Spawners => {
                template.triggers.push(UnitTrigger::Kill(UnitKillTrigger {
                    damage_type: None,
                    effect: Effect::Spawn(Box::new(SpawnEffect {
                        unit_type: "critter".to_owned(),
                    })),
                }));
                let big_critter_percent = 10.0;
                template.walk_effects_mut(&mut |effect| match effect {
                    Effect::Spawn(spawn) => {
                        if spawn.unit_type == "critter" {
                            *effect = Effect::Random {
                                choices: vec![
                                    WeighedEffect {
                                        weight: 100.0 - big_critter_percent,
                                        effect: Effect::Spawn(Box::new(SpawnEffect {
                                            unit_type: "critter".to_owned(),
                                        })),
                                    },
                                    WeighedEffect {
                                        weight: big_critter_percent,
                                        effect: Effect::Spawn(Box::new(SpawnEffect {
                                            unit_type: "big_critter".to_owned(),
                                        })),
                                    },
                                ],
                            }
                        }
                    }
                    _ => {}
                });
                template
                    .triggers
                    .push(UnitTrigger::Spawn(Effect::AddStatus(Box::new(
                        AddStatusEffect {
                            who: Who::Caster,
                            status: Status::Aura(Aura {
                                distance: None,
                                alliance: Some(Alliance::Critters),
                                status: Box::new(Status::Modifier(Modifier::Strength(
                                    StrengthModifier {
                                        multiplier: r32(1.0),
                                        add: r32(2.0),
                                    },
                                ))),
                                time: None,
                            }),
                        },
                    ))));
            }
            Self::Critters => {}
        }
    }
}
