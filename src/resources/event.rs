use strum_macros::AsRefStr;

use super::*;

#[derive(Debug, AsRefStr)]
pub enum Event {
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
        attacker: legion::Entity,
        damage: usize,
    },
    AfterIncomingDamage {
        owner: legion::Entity,
        attacker: legion::Entity,
        damage: usize,
    },
    BeforeOutgoingDamage {
        owner: legion::Entity,
        target: legion::Entity,
        damage: usize,
    },
    AfterOutgoingDamage {
        owner: legion::Entity,
        target: legion::Entity,
        damage: usize,
    },
    AfterDamageDealt {
        owner: legion::Entity,
        target: legion::Entity,
        damage: usize,
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
                attacker,
                damage,
            }
            | Event::AfterIncomingDamage {
                owner,
                attacker,
                damage,
            } => {
                write!(
                    f,
                    "Event {} o:{owner:?} a:{attacker:?} dmg:{damage}",
                    self.as_ref(),
                )
            }

            Event::BeforeOutgoingDamage {
                owner,
                target,
                damage,
            }
            | Event::AfterOutgoingDamage {
                owner,
                target,
                damage,
            }
            | Event::AfterDamageDealt {
                owner,
                target,
                damage,
            } => {
                write!(
                    f,
                    "Event {} o:{owner:?} t:{target:?} dmg:{damage}",
                    self.as_ref(),
                )
            }
            Event::BeforeDeath { owner }
            | Event::AfterDeath { owner }
            | Event::AfterBirth { owner }
            | Event::Buy { owner }
            | Event::Sell { owner }
            | Event::AddToTeam { owner }
            | Event::RemoveFromTeam { owner } => write!(f, "{} o:{:?}", self.as_ref(), owner),
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
            Event::UnitDeath { target } => write!(f, "Event {} t:{:?}", self.as_ref(), target),
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
                Event::UnitDeath { target } => {
                    let factions = Faction::battle();
                    context = context.set_target(*target);
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
                    attacker,
                    damage,
                }
                | Event::AfterIncomingDamage {
                    owner,
                    attacker,
                    damage,
                } => {
                    let context = context
                        .clone_stack(
                            ContextLayer::Var {
                                var: VarName::Damage,
                                value: Var::Int(*damage as i32),
                            },
                            world,
                            resources,
                        )
                        .set_attacker(*attacker);
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
                }
                | Event::BeforeOutgoingDamage {
                    owner,
                    target,
                    damage,
                }
                | Event::AfterDamageDealt {
                    owner,
                    target,
                    damage,
                } => {
                    let context = context
                        .clone_stack(
                            ContextLayer::Var {
                                var: VarName::Damage,
                                value: Var::Int(*damage as i32),
                            },
                            world,
                            resources,
                        )
                        .set_target(*target);
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
