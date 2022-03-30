use super::*;

pub fn initialize(logic: &mut Logic, party_members: usize) {
    for template in logic.model.unit_templates.values_mut() {
        if !template.alliances.contains(&Alliance::Healers) {
            continue;
        }
        if party_members >= 2 {
            template.walk_effects_mut(&mut |effect| {
                if let Effect::Heal(effect) = effect {
                    effect.value = Expr::Mul {
                        a: Box::new(effect.value.clone()),
                        b: Box::new(Expr::Const { value: r32(1.25) }),
                    };
                }
            });
        }
        if party_members >= 4 {
            template.walk_effects_mut(&mut |effect| {
                let p = 0.1;
                if let Effect::Heal(_) = effect {
                    *effect = Effect::List(Box::new(ListEffect {
                        effects: vec![
                            effect.clone(),
                            Effect::Random(Box::new(RandomEffect {
                                choices: vec![
                                    WeightedEffect {
                                        weight: p,
                                        effect: Effect::AttachStatus(Box::new(
                                            AttachStatusEffect {
                                                who: Who::Target,
                                                status: Status::Shield(Box::new(ShieldStatus {})),
                                                time: None,
                                            },
                                        )),
                                    },
                                    WeightedEffect {
                                        weight: 1.0 - p,
                                        effect: Effect::noop(),
                                    },
                                ],
                            })),
                        ],
                    }));
                }
            });
        }
    }
    if party_members >= 6 {
        logic.model.free_revives += 1;
    }
}
