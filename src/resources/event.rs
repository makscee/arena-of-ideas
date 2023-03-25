use super::*;

#[derive(Debug)]
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
    TurnStart,
    TurnEnd,
}

impl Event {
    pub fn send(&self, world: &legion::World, resources: &mut Resources) {
        match self {
            // Send event to every active status
            Event::BattleEnd | Event::BattleStart | Event::TurnStart | Event::TurnEnd => {
                StatusPool::notify_all(self, resources, world);
            }
            // Trigger owner status with owner context
            Event::StatusAdd { status, owner }
            | Event::StatusRemove { status, owner }
            | Event::StatusChargeAdd { status, owner }
            | Event::StatusChargeRemove { status, owner } => {
                let context = ContextSystem::get_context(*owner, world)
                    .add_var(VarName::StatusName, Var::String((0, status.clone())))
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
            }
            // Trigger context.target with provided context
            Event::BeforeIncomingDamage { context } | Event::AfterIncomingDamage { context } => {
                StatusPool::notify_entity(
                    self,
                    context.target,
                    resources,
                    world,
                    Some(context.clone()),
                );
            }
            // Trigger context.owner with provided context
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
                );
            }
            Event::BeforeDeath { owner }
            | Event::AfterBirth { owner }
            | Event::Buy { owner }
            | Event::Sell { owner }
            | Event::AddToTeam { owner }
            | Event::RemoveFromTeam { owner } => {
                StatusPool::notify_entity(self, *owner, resources, world, None);
            }
            Event::BeforeStrike { owner, target }
            | Event::AfterStrike { owner, target }
            | Event::AfterKill { owner, target } => {
                if let Some(owner_context) = ContextSystem::try_get_context(*owner, world).ok() {
                    let context = Context {
                        target: *target,
                        ..owner_context
                    };
                    StatusPool::notify_entity(self, *owner, resources, world, Some(context));
                }
            }
            Event::ModifyOutgoingDamage { .. }
            | Event::ModifyIncomingDamage { .. }
            | Event::ModifyContext { .. } => {
                panic!("Can't send event {:?}", self)
            }
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
