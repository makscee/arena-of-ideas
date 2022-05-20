use super::*;

pub fn initialize(logic: &mut Logic, party_members: usize) {
    for template in logic.model.unit_templates.values_mut() {
        if !template.clans.contains(&Clan::Vampires) {
            continue;
        }
        if party_members >= 2 {
            template.walk_effects_mut(&mut |effect| {
                if let Effect::Damage(effect) = effect {
                    let effect = effect.on.entry(DamageTrigger::Injure).or_default();
                    *effect = Effect::List(Box::new(ListEffect {
                        effects: vec![
                            effect.clone(),
                            Effect::ChangeContext(Box::new(ChangeContextEffect {
                                caster: None,
                                from: Some(Who::Target),
                                target: Some(Who::Caster),
                                effect: Effect::Heal(Box::new(HealEffect {
                                    value: Expr::Mul {
                                        a: Box::new(Expr::Var {
                                            name: VarName::DamageDealt,
                                        }),
                                        b: Box::new(Expr::Const { value: r32(0.2) }),
                                    },
                                    heal_past_max: if party_members >= 4 {
                                        Some(Expr::Mul {
                                            a: Box::new(Expr::FindStat {
                                                who: Who::Caster,
                                                stat: UnitStat::MaxHealth,
                                            }),
                                            b: Box::new(Expr::Const { value: r32(0.4) }),
                                        })
                                    } else {
                                        None
                                    },
                                    add_max_hp: None,
                                })),
                            })),
                        ],
                    }));
                }
            });
        }
        if party_members >= 6 {
            template
                .statuses
                .push(Status::OnKill(Box::new(OnKillStatus {
                    damage_type: None,
                    effect: Effect::Heal(Box::new(HealEffect {
                        value: Expr::Const { value: R32::ZERO },
                        heal_past_max: None,
                        add_max_hp: Some(Expr::Const { value: r32(1.0) }),
                    })),
                })))
        }
    }
}
