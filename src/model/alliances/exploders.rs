use super::*;

pub fn initialize(logic: &mut Logic, party_members: usize) {
    for template in logic.model.unit_templates.values_mut() {
        if !template.alliances.contains(&Alliance::Exploders) {
            continue;
        }
        template.walk_effects_mut(&mut |effect| {
            if let Effect::AOE(aoe) = effect {
                if let Effect::Damage(damage) = &aoe.effect {
                    if party_members >= 2 {
                        aoe.range *= Coord::new(1.4);
                    }
                    if party_members >= 4 {
                        let heal_effect = Effect::AOE(Box::new(AoeEffect {
                            filter: TargetFilter::Allies,
                            skip_current_target: false,
                            range: aoe.range,
                            effect: Effect::Heal(Box::new(HealEffect {
                                value: Expr::Mul {
                                    a: Box::new(damage.value.clone()),
                                    b: Box::new(Expr::Const { value: r32(0.3) }),
                                },
                                heal_past_max: None,
                                add_max_hp: None,
                            })),
                        }));
                        *effect = Effect::List(Box::new(ListEffect {
                            effects: vec![effect.clone(), heal_effect],
                        }));
                    }
                    if party_members >= 6 {
                        *effect = Effect::Random(Box::new(RandomEffect {
                            choices: vec![
                                WeightedEffect {
                                    weight: 80.0,
                                    effect: effect.clone(),
                                },
                                WeightedEffect {
                                    weight: 20.0,
                                    effect: Effect::List(Box::new(ListEffect {
                                        effects: vec![effect.clone(), effect.clone()],
                                    })),
                                },
                            ],
                        }));
                    }
                }
            }
        });
    }
}
