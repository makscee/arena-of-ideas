use super::*;

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum Trigger {
    Init { effect: Effect },
    ModifyIncomingDamage { value: ExpressionInt },
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
        logger: &Logger,
    ) {
        match self {
            Trigger::BeforeIncomingDamage { .. } => match event {
                Event::BeforeIncomingDamage { .. } => self.fire(action_queue, context, logger),
                _ => {}
            },
            Trigger::AfterIncomingDamage { .. } => match event {
                Event::AfterIncomingDamage { .. } => self.fire(action_queue, context, logger),
                _ => {}
            },
            Trigger::List { triggers } => {
                triggers.iter().for_each(|trigger| {
                    trigger.catch_event(event, action_queue, context.clone(), logger)
                });
            }
            Trigger::Init { .. } => match event {
                Event::Init { .. } => self.fire(action_queue, context, logger),
                _ => {}
            },
            Trigger::Buy { .. } => match event {
                Event::Buy { .. } => self.fire(action_queue, context, logger),
                _ => {}
            },
            Trigger::Sell { .. } => match event {
                Event::Sell { .. } => self.fire(action_queue, context, logger),
                _ => {}
            },
            Trigger::RemoveFromTeam { .. } => match event {
                Event::RemoveFromTeam { .. } => self.fire(action_queue, context, logger),
                _ => {}
            },
            Trigger::BeforeDeath { .. } => match event {
                Event::BeforeDeath { .. } => self.fire(action_queue, context, logger),
                _ => {}
            },
            Trigger::AfterBattle { .. } => match event {
                Event::BattleOver { .. } => self.fire(action_queue, context, logger),
                _ => {}
            },
            Trigger::ModifyIncomingDamage { .. } => {}
        }
    }

    fn fire(&self, action_queue: &mut VecDeque<Action>, context: Context, logger: &Logger) {
        match self {
            Trigger::BeforeIncomingDamage { effect }
            | Trigger::AfterIncomingDamage { effect }
            | Trigger::BeforeDeath { effect }
            | Trigger::AfterBattle { effect }
            | Trigger::Init { effect }
            | Trigger::RemoveFromTeam { effect }
            | Trigger::Buy { effect }
            | Trigger::Sell { effect } => {
                logger.log(
                    &format!("Caught trigger {:?}, {:?}", self, context),
                    &LogContext::Trigger,
                );
                action_queue.push_back(Action::new(context, effect.clone()))
            }
            Trigger::ModifyIncomingDamage { .. } | Trigger::List { .. } => {
                panic!("Can't fire {:?}", self)
            }
        }
    }
}
