use super::*;

pub fn initialize(logic: &mut Logic, party_members: usize) {
    for template in logic.model.unit_templates.values_mut() {
        if !template.alliances.contains(&Alliance::Freezers) {
            continue;
        }
        if party_members >= 2 {
            template.walk_effects_mut(&mut |effect| match effect {
                Effect::Damage(damage) => {
                    *effect = Effect::MaybeModify(Box::new(MaybeModifyEffect {
                        base_effect: effect.clone(),
                        condition: Condition::UnitHasStatus {
                            who: Who::Target,
                            status_type: StatusType::Freeze,
                        },
                        modifier: Modifier::Strength(StrengthModifier {
                            value: Expr::Mul {
                                a: Box::new(Expr::Var {
                                    name: VarName::Value,
                                }),
                                b: Box::new(Expr::Const { value: r32(2.0) }),
                            },
                        }),
                    }))
                }
                _ => {}
            })
        }

        if party_members >= 4 {
            template.statuses.push(AttachedStatus {
                status: Status::OnTakeDamage(Box::new(OnTakeDamageStatus {
                    damage_type: None,
                    effect: Effect::AttachStatus(Box::new(AttachStatusEffect {
                        who: Who::Caster,
                        status: AttachedStatus {
                            status: Status::Freeze(Box::new(FreezeStatus {})),
                            time: None,
                        },
                    })),
                })),
                time: None,
            });
        }

        if party_members >= 6 {
            template.statuses.push(AttachedStatus {
                status: Status::OnKill(Box::new(OnKillStatus {
                    damage_type: None,
                    effect: Effect::If(Box::new(IfEffect {
                        condition: Condition::UnitHasStatus {
                            who: Who::Target,
                            status_type: StatusType::Freeze,
                        },
                        then: {
                            Effect::AOE(Box::new(AoeEffect {
                                filter: TargetFilter::Enemies,
                                skip_current_target: true,
                                range: r32(0.5),
                                effect: Effect::Projectile(Box::new(ProjectileEffect {
                                    speed: r32(10.0),
                                    effect: Effect::Damage(Box::new(DamageEffect {
                                        value: Expr::Const { value: r32(1.0) },
                                        types: {
                                            let mut types = HashSet::new();
                                            types.insert("Ranged".to_owned());
                                            types
                                        },
                                        on: HashMap::new(),
                                    })),
                                })),
                            }))
                        },
                        r#else: Effect::noop(),
                    })),
                })),
                time: None,
            });
        }
    }
}
