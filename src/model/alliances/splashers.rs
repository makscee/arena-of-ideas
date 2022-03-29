use super::*;

pub fn initialize(logic: &mut Logic, party_members: usize) {
    for template in logic.model.unit_templates.values_mut() {
        if !template.alliances.contains(&Alliance::Splashers) {
            continue;
        }
        if party_members >= 2 {
            template.action.range *= r32(1.20);
        }
        if party_members >= 4 {
            template.action.effect = Effect::List(Box::new(ListEffect {
                effects: vec![
                    template.action.effect.clone(),
                    Effect::Action(Box::new(ActionEffect {
                        time: Time::new(0.05),
                    })),
                ],
            }));
        }
        if party_members >= 6 {
            template.action.effect.walk_mut(&mut |effect| {
                if let Effect::Splash(splash) = effect {
                    splash.effect_on_caster = Effect::List(Box::new(ListEffect {
                        effects: vec![
                            splash.effect_on_caster.clone(),
                            Effect::NextActionModifier(Box::new(NextActionModifierEffect {
                                modifier: Modifier::Strength(StrengthModifier {
                                    value: Expr::Mul {
                                        a: Box::new(Expr::Var {
                                            name: VarName::Value,
                                        }),
                                        b: Box::new(Expr::Sum {
                                            a: Box::new(Expr::Mul {
                                                a: Box::new(Expr::Var {
                                                    name: VarName::TargetCount,
                                                }),
                                                b: Box::new(Expr::Const { value: r32(0.05) }),
                                            }),
                                            b: Box::new(Expr::Const { value: r32(1.0) }),
                                        }),
                                    },
                                }),
                            })),
                        ],
                    }));
                }
            });
        }
    }
}
