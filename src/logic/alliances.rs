use super::*;

impl Alliance {
    pub fn apply(&self, template: &mut UnitTemplate, party_members: usize) {
        match self {
            Self::Assassins => {
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
                                                    effects.push(Effect::Damage(Box::new(
                                                        DamageEffect {
                                                            value: Expr::Mul {
                                                                a: Box::new(damage.value.clone()),
                                                                b: Box::new(Expr::Const {
                                                                    value: r32(3.0),
                                                                }),
                                                            },
                                                            types: {
                                                                let mut types =
                                                                    damage.types.clone();
                                                                types.insert("Crit".to_owned());
                                                                types
                                                            },
                                                            on: damage.on.clone(),
                                                        },
                                                    )));
                                                    if party_members >= 4 {
                                                        effects.push(Effect::AttachStatus(
                                                            Box::new(AttachStatusEffect {
                                                                who: Who::Target,
                                                                status: AttachedStatus {
                                                                    status: Status::Slow(Box::new(
                                                                        SlowStatus {
                                                                            percent: 70.0,
                                                                        },
                                                                    )),
                                                                    time: Some(r32(3.0)),
                                                                },
                                                            }),
                                                        ));
                                                    }
                                                    if party_members >= 6 {
                                                        effects.push(Effect::AttachStatus(
                                                            Box::new(AttachStatusEffect {
                                                                who: Who::Caster,
                                                                status: AttachedStatus {
                                                                    status: Status::Shield(
                                                                        Box::new(ShieldStatus {}),
                                                                    ),
                                                                    time: None,
                                                                },
                                                            }),
                                                        ));
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
            Self::Spawners => {
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
                                    multiplier: r32(1.0),
                                    add: r32(2.0),
                                }),
                            }))),
                        })),
                        time: None,
                    });
                }
            }
            Self::Archers => {
                template.walk_effects_mut(&mut |effect| match effect {
                    Effect::Projectile(projectile) => {
                        *effect = if party_members >= 6 {
                            Effect::AddTargets(Box::new(AddTargetsEffect {
                                effect: effect.clone(),
                                condition: Condition::Always,
                                additional_targets: None,
                            }))
                        } else if party_members >= 4 {
                            Effect::AddTargets(Box::new(AddTargetsEffect {
                                effect: effect.clone(),
                                condition: Condition::Always,
                                additional_targets: Some(4),
                            }))
                        } else if party_members >= 2 {
                            Effect::AddTargets(Box::new(AddTargetsEffect {
                                effect: effect.clone(),
                                condition: Condition::Always,
                                additional_targets: Some(2),
                            }))
                        } else {
                            effect.clone()
                        };
                    }
                    _ => {}
                });
            }
            Self::Critters => {}
            Self::Freezers => {
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
                                    multiplier: r32(2.0),
                                    add: R32::ZERO,
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
                                        radius: r32(0.5),
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
            Self::Warriors => {
                let mut protection = 0.0;
                if party_members >= 3 {
                    protection = 30.0;
                } else if party_members >= 6 {
                    protection = 50.0;
                }
                if protection != 0.0 {
                    template.statuses.push(AttachedStatus {
                        status: Status::Protection(Box::new(ProtectionStatus {
                            percent: protection,
                        })),
                        time: None,
                    });
                }
            }
            Self::Healers => {
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
                                                        status: AttachedStatus {
                                                            status: Status::Shield(Box::new(
                                                                ShieldStatus {},
                                                            )),
                                                            time: None,
                                                        },
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
                if party_members >= 6 {
                    // TODO
                }
            }
            Self::Vampires => {
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
                                                    a: Box::new(Expr::FindMaxHealth {
                                                        who: Who::Caster,
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
                    template.statuses.push(AttachedStatus {
                        time: None,
                        status: Status::OnKill(Box::new(OnKillStatus {
                            damage_type: None,
                            effect: Effect::Heal(Box::new(HealEffect {
                                value: Expr::Const { value: R32::ZERO },
                                heal_past_max: None,
                                add_max_hp: Some(Expr::Const { value: r32(1.0) }),
                            })),
                        })),
                    })
                }
            }
            Self::Exploders => {
                template.walk_effects_mut(&mut |effect| {
                    if let Effect::AOE(aoe) = effect {
                        if party_members >= 2 {
                            aoe.radius *= Coord::new(1.4);
                        }
                        if party_members >= 4 {
                            // Seems like Explode needs to be a separate effect to count the damage done
                            aoe.effect = Effect::List(Box::new(ListEffect {
                                effects: vec![aoe.effect.clone(), todo!()],
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
                });
            }
        }
    }
}
