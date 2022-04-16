use super::*;

pub fn initialize(logic: &mut Logic, party_members: usize) {
    for template in logic.model.unit_templates.values_mut() {
        if !template.alliances.contains(&Alliance::Charmers) {
            continue;
        }

        template.walk_effects_mut(&mut |effect| match effect {
            Effect::AttachStatus(attach) => match &attach.status {
                Status::Charmed(charmed) => {
                    if party_members >= 2 {
                        if let Some(time) = &mut attach.time {
                            *time *= Time::new(1.5); // +50% duration
                        }
                    }

                    if party_members >= 4 {
                        let attach =
                            std::mem::replace(effect, Effect::Noop(Box::new(NoopEffect {})));
                        let attach_effect = match &attach {
                            Effect::AttachStatus(attach) => attach,
                            _ => unreachable!(),
                        };
                        // +100% damage
                        let strength_modifier = AttachStatusEffect {
                            status: Status::Modifier(Box::new(ModifierStatus {
                                modifier: Modifier::Strength(StrengthModifier {
                                    value: Expr::Mul {
                                        a: Box::new(Expr::Var {
                                            name: VarName::Value,
                                        }),
                                        b: Box::new(Expr::Const { value: r32(2.0) }),
                                    },
                                }),
                            })),
                            who: attach_effect.who,
                            time: attach_effect.time,
                        };
                        let mut effects = Vec::with_capacity(3);

                        if party_members >= 6 {
                            // 5% chance to Charm
                            effects.push(Effect::Random(Box::new(RandomEffect {
                                choices: vec![
                                    WeightedEffect {
                                        weight: 0.95,
                                        effect: Effect::Noop(Box::new(NoopEffect {})),
                                    },
                                    WeightedEffect {
                                        weight: 0.05,
                                        effect: Effect::AttachStatus(Box::new(
                                            AttachStatusEffect {
                                                status: Status::OnDealDamage(Box::new(
                                                    OnDealDamageStatus {
                                                        damage_type: None,
                                                        effect: Effect::AttachStatus(Box::new(
                                                            AttachStatusEffect {
                                                                who: Who::Target,
                                                                status: Status::Charmed(Box::new(
                                                                    CharmedStatus {},
                                                                )),
                                                                time: Some(r32(3.0)), // TODO: remove hardcoded value
                                                            },
                                                        )),
                                                    },
                                                )),
                                                who: attach_effect.who,
                                                time: attach_effect.time,
                                            },
                                        )),
                                    },
                                ],
                            })));
                        }

                        effects.push(Effect::AttachStatus(Box::new(strength_modifier)));
                        effects.push(attach);
                        let list = Effect::List(Box::new(ListEffect { effects }));
                        *effect = list;
                    }
                }
                _ => {}
            },
            _ => {}
        });
    }
}
