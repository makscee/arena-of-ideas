use super::*;

pub enum Event {
    BeforeIncomingDamage,
}

impl Event {
    pub fn send(&self, context: &Context, resources: &mut Resources) -> Result<(), Error> {
        match self {
            Event::BeforeIncomingDamage => {
                resources
                    .statuses
                    .active_statuses
                    .get(&context.target)
                    .unwrap_or(&HashMap::default())
                    .iter()
                    .map(|(status_name, status_context)| {
                        (
                            &resources
                                .statuses
                                .defined_statuses
                                .get(status_name)
                                .expect("Failed to find defined status")
                                .triggers,
                            status_context,
                        )
                    })
                    .for_each(|(triggers, status_context)| {
                        triggers.iter().for_each(|trigger| {
                            trigger.catch_event(
                                self,
                                &mut resources.action_queue,
                                status_context.clone(),
                            )
                        })
                    });
            }
        }
        Ok(())
    }
}
