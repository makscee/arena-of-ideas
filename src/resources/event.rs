use super::*;

#[derive(Debug)]
pub enum Event {
    Init { status: String },
    BeforeIncomingDamage,
    AfterIncomingDamage,
    Buy,
    Sell,
    RemoveFromTeam,
}

impl Event {
    pub fn send(&self, context: &Context, resources: &mut Resources) {
        debug!("Send event {:?} {:?}", self, context);
        match self {
            Event::BeforeIncomingDamage
            | Event::AfterIncomingDamage
            | Event::Buy
            | Event::Sell
            | Event::RemoveFromTeam => {
                resources
                    .status_pool
                    .active_statuses
                    .get(&context.target)
                    .unwrap_or(&HashMap::default())
                    .iter()
                    .map(|(status_name, status_context)| {
                        (
                            &resources
                                .status_pool
                                .defined_statuses
                                .get(status_name)
                                .expect("Failed to find defined status")
                                .trigger,
                            status_context,
                        )
                    })
                    .for_each(|(trigger, status_context)| {
                        trigger.catch_event(self, &mut resources.action_queue, {
                            let mut context = context.clone();
                            context.vars.merge(&status_context.vars, false);
                            context
                        })
                    });
            }
            Event::Init { status } => {
                resources
                    .status_pool
                    .defined_statuses
                    .get(status)
                    .expect("Failed to find defined status for initialization")
                    .trigger
                    .catch_event(self, &mut resources.action_queue, context.clone());
            }
        }
    }
}
