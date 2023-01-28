use super::*;

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum Trigger {
    Init { effect: Effect },
    BeforeIncomingDamage { effect: Effect },
    AfterIncomingDamage { effect: Effect },
    List { triggers: Vec<Box<Trigger>> },
}

impl Trigger {
    /// Link triggers to events
    pub fn catch_event(
        &self,
        event: &Event,
        action_queue: &mut VecDeque<Action>,
        context: Context,
    ) {
        match self {
            Trigger::BeforeIncomingDamage { effect: _ } => match event {
                Event::BeforeIncomingDamage => self.fire(action_queue, context),
                _ => {}
            },
            Trigger::AfterIncomingDamage { effect: _ } => match event {
                Event::AfterIncomingDamage => self.fire(action_queue, context),
                _ => {}
            },
            Trigger::List { triggers } => {
                triggers
                    .iter()
                    .for_each(|trigger| trigger.catch_event(event, action_queue, context.clone()));
            }
            Trigger::Init { effect: _ } => match event {
                Event::Init { status: _ } => self.fire(action_queue, context),
                _ => {}
            },
        }
    }

    fn fire(&self, action_queue: &mut VecDeque<Action>, context: Context) {
        match self {
            Trigger::BeforeIncomingDamage { effect }
            | Trigger::AfterIncomingDamage { effect }
            | Trigger::Init { effect } => {
                action_queue.push_back(Action::new(context, effect.clone()))
            }
            Trigger::List { triggers: _ } => todo!(),
        }
    }
}
