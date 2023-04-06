use strum_macros::AsRefStr;

use super::*;

#[derive(Debug, AsRefStr)]
pub enum Event {
    StatusAdd {
        status: String,
        owner: legion::Entity,
    },
    StatusRemove {
        status: String,
        owner: legion::Entity,
    },
    StatusChargeAdd {
        status: String,
        owner: legion::Entity,
    },
    StatusChargeRemove {
        status: String,
        owner: legion::Entity,
    },
    ModifyIncomingDamage {
        context: Context,
    },
    ModifyOutgoingDamage {
        context: Context,
    },
    ModifyContext {
        context: Context,
    },
    BeforeIncomingDamage {
        context: Context,
    },
    AfterIncomingDamage {
        context: Context,
    },
    BeforeOutgoingDamage {
        context: Context,
    },
    AfterOutgoingDamage {
        context: Context,
    },
    AfterDamageDealt {
        context: Context,
    },
    AfterDeath {
        context: Context,
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
            Event::StatusAdd { status, owner }
            | Event::StatusRemove { status, owner }
            | Event::StatusChargeAdd { status, owner }
            | Event::StatusChargeRemove { status, owner } => {
                write!(f, "{} {} o:{:?}", self.as_ref(), status, owner)
            }
            Event::ModifyIncomingDamage { context }
            | Event::ModifyOutgoingDamage { context }
            | Event::ModifyContext { context }
            | Event::BeforeIncomingDamage { context }
            | Event::AfterIncomingDamage { context }
            | Event::BeforeOutgoingDamage { context }
            | Event::AfterOutgoingDamage { context }
            | Event::AfterDamageDealt { context }
            | Event::AfterDeath { context } => {
                write!(
                    f,
                    "{} o:{:?} t:{:?}",
                    self.as_ref(),
                    context.owner,
                    context.target
                )
            }
            Event::BeforeDeath { owner }
            | Event::AfterBirth { owner }
            | Event::Buy { owner }
            | Event::Sell { owner }
            | Event::AddToTeam { owner }
            | Event::RemoveFromTeam { owner } => write!(f, "{} o:{:?}", self.as_ref(), owner),
            Event::BeforeStrike { owner, target }
            | Event::AfterStrike { owner, target }
            | Event::AfterKill { owner, target } => {
                write!(f, "{} o:{:?} t:{:?}", self.as_ref(), owner, target)
            }
            Event::BattleStart
            | Event::BattleEnd
            | Event::ShopStart
            | Event::ShopEnd
            | Event::TurnStart
            | Event::TurnEnd => write!(f, "{}", self.as_ref()),
            Event::UnitDeath { target } => write!(f, "{} t:{:?}", self.as_ref(), target),
        }
    }
}

impl Event {
    pub fn send(&self, world: &legion::World, resources: &mut Resources) {
        let mut caught = false;
        // Notify all Faction::Dark and Faction::Light
        caught = caught
            || match self {
                Event::BattleEnd | Event::BattleStart | Event::TurnStart | Event::TurnEnd => {
                    StatusPool::notify_all(self, &Faction::battle(), resources, world, None);
                    true
                }
                _ => false,
            };

        // Notify all Faction::Dark and Faction::Light and set target
        caught = caught
            || match self {
                Event::UnitDeath { target } => {
                    StatusPool::notify_all(
                        self,
                        &Faction::battle(),
                        resources,
                        world,
                        Some(*target),
                    );
                    true
                }
                _ => false,
            };

        // Notify Faction::Team
        caught = caught
            || match self {
                Event::ShopStart | Event::ShopEnd | Event::Sell { .. } | Event::Buy { .. } => {
                    let factions = hashset! {Faction::Team};
                    StatusPool::notify_all(self, &factions, resources, world, None);
                    true
                }
                _ => false,
            };

        // Notify specified status of owner
        caught = caught
            || match self {
                Event::StatusAdd { status, owner }
                | Event::StatusRemove { status, owner }
                | Event::StatusChargeAdd { status, owner }
                | Event::StatusChargeRemove { status, owner } => {
                    let context = ContextSystem::get_context(*owner, world)
                        .add_var(VarName::StatusName, Var::String((1, status.clone())))
                        .to_owned();
                    resources
                        .status_pool
                        .defined_statuses
                        .get(status)
                        .unwrap()
                        .trigger
                        .catch_event(
                            self,
                            &mut resources.action_queue,
                            context,
                            &resources.logger,
                        );
                    true
                }
                _ => false,
            };

        // Notify context.target with provided context
        caught = caught
            || match self {
                Event::BeforeIncomingDamage { context }
                | Event::AfterIncomingDamage { context } => {
                    StatusPool::notify_entity(
                        self,
                        context.target,
                        resources,
                        world,
                        Some(context.clone()),
                        None,
                    );
                    true
                }
                _ => false,
            };

        // Notify context.owner with provided context
        caught = caught
            || match self {
                Event::AfterOutgoingDamage { context }
                | Event::BeforeOutgoingDamage { context }
                | Event::AfterDeath { context }
                | Event::AfterDamageDealt { context } => {
                    StatusPool::notify_entity(
                        self,
                        context.owner,
                        resources,
                        world,
                        Some(context.clone()),
                        None,
                    );
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
                    if let Some(owner_context) = ContextSystem::try_get_context(*owner, world).ok()
                    {
                        let context = Context {
                            target: *target,
                            ..owner_context
                        };
                        StatusPool::notify_entity(
                            self,
                            *owner,
                            resources,
                            world,
                            Some(context),
                            None,
                        );
                    }
                    true
                }
                _ => false,
            };

        // Notify owner
        caught = caught
            || match self {
                Event::BeforeDeath { owner }
                | Event::AfterBirth { owner }
                | Event::Buy { owner }
                | Event::Sell { owner }
                | Event::AddToTeam { owner }
                | Event::RemoveFromTeam { owner } => {
                    StatusPool::notify_entity(self, *owner, resources, world, None, None);
                    true
                }
                _ => false,
            };
        if !caught {
            panic!("{:?} was never caught", self)
        }
    }

    pub fn calculate(&self, world: &legion::World, resources: &Resources) -> Context {
        match self {
            Event::ModifyContext { context } | Event::ModifyIncomingDamage { context } => {
                StatusPool::calculate_entity(
                    &self,
                    context.target,
                    context.clone(),
                    world,
                    resources,
                )
            }
            Event::ModifyOutgoingDamage { context } => StatusPool::calculate_entity(
                &self,
                context.owner,
                context.clone(),
                world,
                resources,
            ),
            _ => {
                panic!("Can't calculate event {:?}", self)
            }
        }
    }
}
