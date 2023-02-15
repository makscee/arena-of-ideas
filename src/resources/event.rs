use super::*;

pub enum Event {
    Init { status: String },
    BeforeIncomingDamage,
    AfterIncomingDamage,
}

impl Event {
    pub fn send(&self, context: &Context, resources: &mut Resources) -> Result<(), Error> {
        match self {
            Event::BeforeIncomingDamage | Event::AfterIncomingDamage => {
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
                        trigger.catch_event(
                            self,
                            &mut resources.action_queue,
                            status_context.clone(),
                        )
                    });
            }
            Event::Init { status } => {
                resources
                    .status_pool
                    .defined_statuses
                    .get(status)
                    .context("Failed to find defined status for initialization")?
                    .trigger
                    .catch_event(self, &mut resources.action_queue, context.clone());
            }
        }
        Ok(())
    }
}
