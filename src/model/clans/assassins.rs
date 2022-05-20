use super::*;

pub fn initialize(logic: &mut Logic, party_members: usize) {
    for template in logic.model.unit_templates.values_mut() {
        if !template.clans.contains(&Clan::Assassins) {
            continue;
        }
        if party_members >= 2 {
            let crit_percent = 15.0;
            template
                .action
                .effect
                .walk_mut(&mut |effect| match &effect {
                    Effect::Damage(damage) => {
                        *effect = Effect::Random(Box::new(RandomEffect {
                            choices: vec![
                                WeightedEffect {
                                    weight: 100.0 - crit_percent,
                                    effect: effect.clone(),
                                },
                                WeightedEffect {
                                    weight: crit_percent,
                                    effect: Effect::List(Box::new(ListEffect {
                                        effects: {
                                            let mut effects = Vec::new();
                                            effects.push(Effect::Damage(Box::new(DamageEffect {
                                                value: Expr::Mul {
                                                    a: Box::new(damage.value.clone()),
                                                    b: Box::new(Expr::Const { value: r32(3.0) }),
                                                },
                                                types: {
                                                    let mut types = damage.types.clone();
                                                    types.insert("Crit".to_owned());
                                                    types
                                                },
                                                on: damage.on.clone(),
                                            })));
                                            if party_members >= 4 {
                                                effects.push(Effect::AttachStatus(Box::new(
                                                    AttachStatusEffect {
                                                        who: Who::Target,
                                                        status: Status::Slow(Box::new(
                                                            SlowStatus { percent: 70.0 },
                                                        )),
                                                        time: Some(r32(3.0)),
                                                    },
                                                )));
                                            }
                                            if party_members >= 6 {
                                                effects.push(Effect::AttachStatus(Box::new(
                                                    AttachStatusEffect {
                                                        who: Who::Caster,
                                                        status: Status::Shield(Box::new(
                                                            ShieldStatus {},
                                                        )),
                                                        time: None,
                                                    },
                                                )));
                                            }
                                            effects
                                        },
                                    })),
                                },
                            ],
                        }));
                    }
                    _ => {}
                });
        }
    }
}
