use super::*;

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum Trigger {
    BeforeIncomingDamage { effect: Effect },
    List { triggers: Vec<Box<Trigger>> },
}

impl Trigger {
    pub fn catch_event(
        &self,
        event: &Event,
        action_queue: &mut VecDeque<Action>,
        context: Context,
    ) {
        match self {
            Trigger::BeforeIncomingDamage { effect: _ } => match event {
                Event::BeforeIncomingDamage => self.fire(action_queue, context),
            },
            Trigger::List { triggers } => {
                triggers
                    .iter()
                    .for_each(|trigger| trigger.catch_event(event, action_queue, context.clone()));
            }
        }
    }

    fn fire(&self, action_queue: &mut VecDeque<Action>, context: Context) {
        match self {
            Trigger::BeforeIncomingDamage { effect } => {
                action_queue.push_back(Action::new(context, effect.clone()))
            }
            Trigger::List { triggers: _ } => todo!(),
        }
    }
}
