use strum_macros::AsRefStr;

use super::*;

#[derive(Debug, AsRefStr)]
pub enum Event {
    AbilityUse {
        ability: AbilityName,
        caster: legion::Entity,
        target: legion::Entity,
    },
    StatusAdd {
        status: String,
        owner: legion::Entity,
        charges: i32,
    },
    StatusRemove {
        status: String,
        owner: legion::Entity,
        charges: i32,
    },
    StatusChargeAdd {
        status: String,
        owner: legion::Entity,
        charges: i32,
    },
    StatusChargeRemove {
        status: String,
        owner: legion::Entity,
        charges: i32,
    },
    ModifyIncomingDamage,
    ModifyOutgoingDamage,
    ModifyContext,
    BeforeIncomingDamage {
        owner: legion::Entity,
        caster: legion::Entity,
        damage: usize,
        source: String,
    },
    AfterIncomingDamage {
        owner: legion::Entity,
        caster: legion::Entity,
        damage: usize,
        source: String,
    },
    BeforeOutgoingDamage {
        owner: legion::Entity,
        target: legion::Entity,
        damage: usize,
        source: String,
    },
    AfterOutgoingDamage {
        owner: legion::Entity,
        target: legion::Entity,
        damage: usize,
        source: String,
    },
    AfterDamageDealt {
        owner: legion::Entity,
        target: legion::Entity,
        damage: usize,
        source: String,
    },
    AfterDeath {
        owner: legion::Entity,
    },
    BeforeDeath {
        owner: legion::Entity,
    },
    AfterBirth {
        owner: legion::Entity,
    },
    Buy {
        owner: legion::Entity,
    },
    Sell {
        owner: legion::Entity,
    },
    AddToTeam {
        owner: legion::Entity,
    },
    RemoveFromTeam {
        owner: legion::Entity,
    },
    BeforeStrike {
        owner: legion::Entity,
        target: legion::Entity,
    },
    AfterStrike {
        owner: legion::Entity,
        target: legion::Entity,
    },
    AfterKill {
        owner: legion::Entity,
        target: legion::Entity,
    },
    BattleStart,
    BattleEnd,
    ShopStart,
    ShopEnd,
    TurnStart,
    TurnEnd,
    UnitDeath {
        target: legion::Entity,
        killer: legion::Entity,
    },
}

impl Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Event::StatusAdd {
                status,
                owner,
                charges,
            }
            | Event::StatusRemove {
                status,
                owner,
                charges,
            }
            | Event::StatusChargeAdd {
                status,
                owner,
                charges,
            }
            | Event::StatusChargeRemove {
                status,
                owner,
                charges,
            } => {
                write!(
                    f,
                    "Event {} {status} c:{charges} o:{owner:?}",
                    self.as_ref(),
                )
            }
            Event::BeforeIncomingDamage {
                owner,
                caster,
                damage,
                source,
            }
            | Event::AfterIncomingDamage {
                owner,
                caster,
                damage,
                source,
            } => {
                write!(
                    f,
                    "Event {} o:{owner:?} c:{caster:?} dmg:{damage} s:{source}",
                    self.as_ref(),
                )
            }

            Event::BeforeOutgoingDamage {
                owner,
                target,
                damage,
                source,
            }
            | Event::AfterOutgoingDamage {
                owner,
                target,
                damage,
                source,
            }
            | Event::AfterDamageDealt {
                owner,
                target,
                damage,
                source,
            } => {
                write!(
                    f,
                    "Event {} o:{owner:?} t:{target:?} dmg:{damage} s:{source}",
                    self.as_ref(),
                )
            }
            Event::BeforeDeath { owner }
            | Event::AfterDeath { owner }
            | Event::AfterBirth { owner }
            | Event::Buy { owner }
            | Event::Sell { owner }
            | Event::AddToTeam { owner }
            | Event::RemoveFromTeam { owner } => write!(f, "{} o:{owner:?}", self.as_ref()),
            Event::BeforeStrike { owner, target }
            | Event::AfterStrike { owner, target }
            | Event::AfterKill { owner, target } => {
                write!(f, "Event {} o:{:?} t:{:?}", self.as_ref(), owner, target)
            }
            Event::ModifyIncomingDamage
            | Event::ModifyOutgoingDamage
            | Event::ModifyContext
            | Event::BattleStart
            | Event::BattleEnd
            | Event::ShopStart
            | Event::ShopEnd
            | Event::TurnStart
            | Event::TurnEnd => write!(f, "Event {}", self.as_ref()),
            Event::UnitDeath { target, killer } => {
                write!(f, "Event {} t:{target:?} k:{killer:?}", self.as_ref())
            }
            Event::AbilityUse {
                ability,
                caster,
                target,
            } => write!(
                f,
                "Event {} a:{ability} c:{caster:?} t:{target:?}",
                self.as_ref()
            ),
        }
    }
}

impl Event {
    pub fn send(&self, world: &legion::World, resources: &mut Resources) {
        let mut context = Context::new_empty(&self.to_string());
        resources
            .logger
            .log(|| self.to_string(), &LogContext::Event);
        let mut caught = false;
        // Notify all Faction::Dark and Faction::Light
        caught = caught
            || match self {
                Event::BattleEnd | Event::BattleStart | Event::TurnStart | Event::TurnEnd => {
                    let factions = Faction::battle();
                    Status::notify_all(self, &factions, &context, world, resources);
                    true
                }
                _ => false,
            };

        // Notify all Faction::Dark and Faction::Light and set target
        caught = caught
            || match self {
                Event::UnitDeath { target, killer } => {
                    let factions = Faction::battle();
                    context = context.set_target(*target).set_caster(*killer);
                    Status::notify_all(self, &factions, &context, world, resources);
                    true
                }
                _ => false,
            };

        // Notify all Faction::Dark and Faction::Light, set target & caster
        caught = caught
            || match self {
                Event::AbilityUse { caster, target, .. } => {
                    let factions = Faction::battle();
                    context = context.set_target(*target).set_caster(*caster);
                    Status::notify_all(self, &factions, &context, world, resources);
                    true
                }
                _ => false,
            };

        // Notify Faction::Team
        caught = caught
            || match self {
                Event::ShopStart | Event::ShopEnd | Event::Sell { .. } | Event::Buy { .. } => {
                    let factions = hashset! {Faction::Team};
                    Status::notify_all(self, &factions, &context, world, resources);
                    true
                }
                _ => false,
            };

        // Notify specified status of owner
        caught = caught
            || match self {
                Event::StatusAdd {
                    status,
                    owner,
                    charges,
                }
                | Event::StatusRemove {
                    status,
                    owner,
                    charges,
                }
                | Event::StatusChargeAdd {
                    status,
                    owner,
                    charges,
                }
                | Event::StatusChargeRemove {
                    status,
                    owner,
                    charges,
                } => {
                    Status::notify_status(
                        self, *owner, &context, status, *charges, world, resources,
                    );
                    true
                }
                _ => false,
            };

        // Notify context.target with provided context
        caught = caught
            || match self {
                Event::BeforeIncomingDamage {
                    owner,
                    caster,
                    damage,
                    source,
                }
                | Event::AfterIncomingDamage {
                    owner,
                    caster,
                    damage,
                    source,
                } => {
                    let mut context = context
                        .clone_stack(
                            ContextLayer::Var {
                                var: VarName::Damage,
                                value: Var::Int(*damage as i32),
                            },
                            world,
                            resources,
                        )
                        .set_caster(*caster);
                    context.insert_string(VarName::Source, (0, source.clone()));
                    Status::notify_one(self, *owner, &context, world, resources);
                    true
                }
                _ => false,
            };

        // Notify owner, set target, add damage
        caught = caught
            || match self {
                Event::AfterOutgoingDamage {
                    owner,
                    target,
                    damage,
                    source,
                }
                | Event::BeforeOutgoingDamage {
                    owner,
                    target,
                    damage,
                    source,
                }
                | Event::AfterDamageDealt {
                    owner,
                    target,
                    damage,
                    source,
                } => {
                    let mut context = context
                        .clone_stack(
                            ContextLayer::Var {
                                var: VarName::Damage,
                                value: Var::Int(*damage as i32),
                            },
                            world,
                            resources,
                        )
                        .set_target(*target);
                    context.insert_string(VarName::Source, (0, source.clone()));
                    Status::notify_one(self, *owner, &context, world, resources);
                    true
                }
                _ => false,
            };

        // Notify owner and set target
        caught = caught
            || match self {
                Event::BeforeStrike { owner, target }
                | Event::AfterStrike { owner, target }
                | Event::AfterKill { owner, target } => {
                    let context = context.clone_stack(
                        ContextLayer::Target { entity: *target },
                        world,
                        resources,
                    );
                    Status::notify_one(self, *owner, &context, world, resources);
                    true
                }
                _ => false,
            };

        // Notify owner
        caught = caught
            || match self {
                Event::BeforeDeath { owner }
                | Event::AfterDeath { owner }
                | Event::AfterBirth { owner }
                | Event::Buy { owner }
                | Event::Sell { owner }
                | Event::AddToTeam { owner }
                | Event::RemoveFromTeam { owner } => {
                    Status::notify_one(self, *owner, &context, world, resources);
                    true
                }
                _ => false,
            };
        if !caught {
            panic!("{:?} was never caught", self)
        }
    }

    pub fn calculate(self, context: &mut Context, world: &legion::World, resources: &Resources) {
        match self {
            Event::ModifyContext | Event::ModifyOutgoingDamage | Event::ModifyIncomingDamage => {
                Status::calculate_one(self, context.owner().unwrap(), context, world, resources)
            }
            _ => {
                panic!("Can't calculate event {:?}", self)
            }
        };
    }
}
