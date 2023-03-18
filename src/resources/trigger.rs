use super::*;

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum Trigger {
    OnStatusAdd { effect: Effect },
    OnStatusRemove { effect: Effect },
    OnStatusChargeAdd { effect: Effect },
    OnStatusChargeRemove { effect: Effect },
    ModifyIncomingDamage { value: ExpressionInt },
    BeforeIncomingDamage { effect: Effect },
    AfterIncomingDamage { effect: Effect },
    BeforeDeath { effect: Effect },
    AfterBattle { effect: Effect },
    List { triggers: Vec<Box<Trigger>> },
    Buy { effect: Effect },
    Sell { effect: Effect },
    AfterStrike { effect: Effect },
    AddToTeam { effect: Effect },
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
            Trigger::Buy { .. } => match event {
                Event::Buy { .. } => self.fire(action_queue, context, logger),
                _ => {}
            },
            Trigger::Sell { .. } => match event {
                Event::Sell { .. } => self.fire(action_queue, context, logger),
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
            Trigger::AfterStrike { .. } => match event {
                Event::AfterStrike { .. } => self.fire(action_queue, context, logger),
                _ => {}
            },
            Trigger::OnStatusAdd { .. } => match event {
                Event::StatusAdd { .. } => self.fire(action_queue, context, logger),
                _ => {}
            },
            Trigger::OnStatusRemove { .. } => match event {
                Event::StatusRemove { .. } => self.fire(action_queue, context, logger),
                _ => {}
            },
            Trigger::OnStatusChargeAdd { .. } => match event {
                Event::StatusChargeAdd { .. } => self.fire(action_queue, context, logger),
                _ => {}
            },
            Trigger::OnStatusChargeRemove { .. } => match event {
                Event::StatusChargeRemove { .. } => self.fire(action_queue, context, logger),
                _ => {}
            },
            Trigger::AddToTeam { .. } => match event {
                Event::AddToTeam { .. } => self.fire(action_queue, context, logger),
                _ => {}
            },
            Trigger::RemoveFromTeam { .. } => match event {
                Event::RemoveFromTeam { .. } => self.fire(action_queue, context, logger),
                _ => {}
            },
        }
    }

    fn fire(&self, action_queue: &mut VecDeque<Action>, context: Context, logger: &Logger) {
        match self {
            Trigger::BeforeIncomingDamage { effect }
            | Trigger::AfterIncomingDamage { effect }
            | Trigger::BeforeDeath { effect }
            | Trigger::AfterBattle { effect }
            | Trigger::AfterStrike { effect }
            | Trigger::Buy { effect }
            | Trigger::Sell { effect }
            | Trigger::AddToTeam { effect }
            | Trigger::RemoveFromTeam { effect }
            | Trigger::OnStatusAdd { effect }
            | Trigger::OnStatusRemove { effect }
            | Trigger::OnStatusChargeAdd { effect }
            | Trigger::OnStatusChargeRemove { effect } => {
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
