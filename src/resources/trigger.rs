use super::*;

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum Trigger {
    Init { effect: Effect },
    BeforeIncomingDamage { effect: Effect },
    AfterIncomingDamage { effect: Effect },
    BeforeDeath { effect: Effect },
    AfterBattle { effect: Effect },
    List { triggers: Vec<Box<Trigger>> },
    Buy { effect: Effect },
    Sell { effect: Effect },
    RemoveFromTeam { effect: Effect },
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
            Trigger::BeforeIncomingDamage { .. } => match event {
                Event::BeforeIncomingDamage { .. } => self.fire(action_queue, context),
                _ => {}
            },
            Trigger::AfterIncomingDamage { .. } => match event {
                Event::AfterIncomingDamage { .. } => self.fire(action_queue, context),
                _ => {}
            },
            Trigger::List { triggers } => {
                triggers
                    .iter()
                    .for_each(|trigger| trigger.catch_event(event, action_queue, context.clone()));
            }
            Trigger::Init { .. } => match event {
                Event::Init { .. } => self.fire(action_queue, context),
                _ => {}
            },
            Trigger::Buy { .. } => match event {
                Event::Buy { .. } => self.fire(action_queue, context),
                _ => {}
            },
            Trigger::Sell { .. } => match event {
                Event::Sell { .. } => self.fire(action_queue, context),
                _ => {}
            },
            Trigger::RemoveFromTeam { .. } => match event {
                Event::RemoveFromTeam { .. } => self.fire(action_queue, context),
                _ => {}
            },
            Trigger::BeforeDeath { .. } => match event {
                Event::BeforeDeath { .. } => self.fire(action_queue, context),
                _ => {}
            },
            Trigger::AfterBattle { .. } => match event {
                Event::AfterBattle { .. } => self.fire(action_queue, context),
                _ => {}
            },
        }
    }

    fn fire(&self, action_queue: &mut VecDeque<Action>, context: Context) {
        match self {
            Trigger::BeforeIncomingDamage { effect }
            | Trigger::AfterIncomingDamage { effect }
            | Trigger::BeforeDeath { effect }
            | Trigger::AfterBattle { effect }
            | Trigger::Init { effect }
            | Trigger::RemoveFromTeam { effect }
            | Trigger::Buy { effect }
            | Trigger::Sell { effect } => {
                action_queue.push_back(Action::new(context, effect.clone()))
            }
            Trigger::List { triggers: _ } => panic!("List should not fire"),
        }
    }
}
