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
    BeforeIncomingDamage {
        context: Context,
    },
    AfterIncomingDamage {
        context: Context,
    },
    BeforeDeath {
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
    AfterStrike {
        owner: legion::Entity,
        target: legion::Entity,
    },
    BattleOver,
}

impl Event {
    pub fn send(&self, resources: &mut Resources, world: &legion::World) -> Option<Context> {
        resources
            .logger
            .log(&format!("Send event {:?}", self), &LogContext::Event);
        match self {
            // Send event to every active status
            Event::BattleOver => {
                StatusPool::notify_all(self, resources, world);
                None
            }
            // Modify Damage var in provided context and return updated context
            Event::ModifyIncomingDamage { context } => {
                let mut context = context.clone();
                let mut damage = context.vars.get_int(&VarName::Damage);
                resources
                    .status_pool
                    .collect_triggers(&context.target)
                    .iter()
                    .for_each(|(name, trigger, charges)| match trigger {
                        Trigger::ModifyIncomingDamage { value } => {
                            damage = match value.calculate(
                                context
                                    .add_var(VarName::Charges, Var::Int(*charges))
                                    .add_var(
                                        VarName::StatusName,
                                        Var::String((0, name.to_string())),
                                    ),
                                world,
                                resources,
                            ) {
                                Ok(value) => value,
                                Err(_) => damage,
                            };
                            context.vars.insert(VarName::Damage, Var::Int(damage));
                        }
                        _ => {}
                    });
                Some(context)
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
                None
            }
            // Trigger context.owner with provided context
            Event::BeforeIncomingDamage { context } | Event::AfterIncomingDamage { context } => {
                StatusPool::notify_entity(
                    self,
                    context.target,
                    resources,
                    world,
                    Some(context.clone()),
                );
                None
            }
            Event::BeforeDeath { owner }
            | Event::Buy { owner }
            | Event::Sell { owner }
            | Event::AddToTeam { owner }
            | Event::RemoveFromTeam { owner } => {
                StatusPool::notify_entity(self, *owner, resources, world, None);
                None
            }
            Event::AfterStrike { owner, target } => {
                let context = Context {
                    target: *target,
                    ..ContextSystem::get_context(*owner, world)
                };
                StatusPool::notify_entity(self, *owner, resources, world, Some(context));
                None
            }
        }
    }
}
