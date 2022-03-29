use super::*;

pub fn initialize(logic: &mut Logic, party_members: usize) {
    for template in logic.model.unit_templates.values_mut() {
        if !template.alliances.contains(&Alliance::Spawners) {
            continue;
        }

        if party_members >= 4 {
            template.statuses.push(AttachedStatus {
                status: Status::OnKill(Box::new(OnKillStatus {
                    damage_type: None,
                    effect: Effect::Spawn(Box::new(SpawnEffect {
                        unit_type: "critter".to_owned(),
                    })),
                })),
                time: None,
            });
        }
        if party_members >= 6 {
            let big_critter_percent = 10.0;
            template.walk_effects_mut(&mut |effect| match effect {
                Effect::Spawn(spawn) => {
                    if spawn.unit_type == "critter" {
                        *effect = Effect::Random(Box::new(RandomEffect {
                            choices: vec![
                                WeightedEffect {
                                    weight: 100.0 - big_critter_percent,
                                    effect: Effect::Spawn(Box::new(SpawnEffect {
                                        unit_type: "critter".to_owned(),
                                    })),
                                },
                                WeightedEffect {
                                    weight: big_critter_percent,
                                    effect: Effect::Spawn(Box::new(SpawnEffect {
                                        unit_type: "big_critter".to_owned(),
                                    })),
                                },
                            ],
                        }));
                    }
                }
                _ => {}
            });
        }
        if party_members >= 2 {
            template.statuses.push(AttachedStatus {
                status: Status::Aura(Box::new(AuraStatus {
                    distance: None,
                    alliance: Some(Alliance::Critters),
                    status: Box::new(Status::Modifier(Box::new(ModifierStatus {
                        modifier: Modifier::Strength(StrengthModifier {
                            value: Expr::Sum {
                                a: Box::new(Expr::Var {
                                    name: VarName::Value,
                                }),
                                b: Box::new(Expr::Const { value: r32(2.0) }),
                            },
                        }),
                    }))),
                })),
                time: None,
            });
        }
    }
}
