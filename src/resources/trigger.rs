use super::*;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "type")]
#[serde(deny_unknown_fields)]
pub enum Trigger {
    List { triggers: Vec<Box<Trigger>> },
    OnStatusAdd { effect: EffectWrapped },
    OnStatusRemove { effect: EffectWrapped },
    OnStatusChargeAdd { effect: EffectWrapped },
    OnStatusChargeRemove { effect: EffectWrapped },
    BeforeIncomingDamage { effect: EffectWrapped },
    AfterIncomingDamage { effect: EffectWrapped },
    BeforeOutgoingDamage { effect: EffectWrapped },
    AfterOutgoingDamage { effect: EffectWrapped },
    AfterDamageDealt { effect: EffectWrapped },
    AfterKill { effect: EffectWrapped },
    BeforeDeath { effect: EffectWrapped },
    AfterDeath { effect: EffectWrapped },
    AfterBirth { effect: EffectWrapped },
    BattleStart { effect: EffectWrapped },
    BattleEnd { effect: EffectWrapped },
    TurnStart { effect: EffectWrapped },
    TurnEnd { effect: EffectWrapped },
    Buy { effect: EffectWrapped },
    Sell { effect: EffectWrapped },
    BeforeStrike { effect: EffectWrapped },
    AfterStrike { effect: EffectWrapped },
    AddToTeam { effect: EffectWrapped },
    RemoveFromTeam { effect: EffectWrapped },
    ModifyIncomingDamage { value: ExpressionInt },
    ModifyOutgoingDamage { value: ExpressionInt },
    ChangeVarInt { var: VarName, delta: ExpressionInt }, // Preferred for stat changes
    Noop,
}

impl Default for Trigger {
    fn default() -> Self {
        Trigger::Noop
    }
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
            Trigger::BeforeOutgoingDamage { .. } => match event {
                Event::BeforeOutgoingDamage { .. } => self.fire(action_queue, context, logger),
                _ => {}
            },
            Trigger::AfterOutgoingDamage { .. } => match event {
                Event::AfterOutgoingDamage { .. } => self.fire(action_queue, context, logger),
                _ => {}
            },
            Trigger::AfterDamageDealt { .. } => match event {
                Event::AfterDamageDealt { .. } => self.fire(action_queue, context, logger),
                _ => {}
            },
            Trigger::AfterKill { .. } => match event {
                Event::AfterKill { .. } => self.fire(action_queue, context, logger),
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
            Trigger::AfterDeath { .. } => match event {
                Event::AfterDeath { .. } => self.fire(action_queue, context, logger),
                _ => {}
            },
            Trigger::AfterBirth { .. } => match event {
                Event::AfterBirth { .. } => self.fire(action_queue, context, logger),
                _ => {}
            },
            Trigger::BattleStart { .. } => match event {
                Event::BattleStart { .. } => self.fire(action_queue, context, logger),
                _ => {}
            },
            Trigger::BattleEnd { .. } => match event {
                Event::BattleEnd { .. } => self.fire(action_queue, context, logger),
                _ => {}
            },
            Trigger::TurnStart { .. } => match event {
                Event::TurnStart { .. } => self.fire(action_queue, context, logger),
                _ => {}
            },
            Trigger::TurnEnd { .. } => match event {
                Event::TurnEnd { .. } => self.fire(action_queue, context, logger),
                _ => {}
            },
            Trigger::BeforeStrike { .. } => match event {
                Event::BeforeStrike { .. } => self.fire(action_queue, context, logger),
                _ => {}
            },
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
            Trigger::ModifyOutgoingDamage { .. }
            | Trigger::ModifyIncomingDamage { .. }
            | Trigger::ChangeVarInt { .. }
            | Trigger::Noop => {}
        }
    }

    fn fire(&self, action_queue: &mut VecDeque<Action>, context: Context, logger: &Logger) {
        match self {
            Trigger::BeforeIncomingDamage { effect }
            | Trigger::AfterIncomingDamage { effect }
            | Trigger::BeforeOutgoingDamage { effect }
            | Trigger::AfterOutgoingDamage { effect }
            | Trigger::AfterDamageDealt { effect }
            | Trigger::AfterKill { effect }
            | Trigger::BeforeDeath { effect }
            | Trigger::AfterDeath { effect }
            | Trigger::AfterBirth { effect }
            | Trigger::BattleStart { effect }
            | Trigger::BattleEnd { effect }
            | Trigger::TurnStart { effect }
            | Trigger::TurnEnd { effect }
            | Trigger::BeforeStrike { effect }
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
            Trigger::ModifyIncomingDamage { .. }
            | Trigger::ModifyOutgoingDamage { .. }
            | Trigger::List { .. }
            | Trigger::ChangeVarInt { .. } => {
                panic!("Can't fire {:?}", self)
            }
            Trigger::Noop => {}
        }
    }

    /// Change vars and return updated context
    pub fn calculate_event(
        &self,
        event: &Event,
        context: Context,
        world: &legion::World,
        resources: &Resources,
    ) -> Context {
        let mut context = context.clone();
        match self {
            Trigger::List { triggers } => {
                for trigger in triggers {
                    context = trigger.calculate_event(event, context, world, resources);
                }
            }
            Trigger::ModifyIncomingDamage { value } => match event {
                Event::ModifyIncomingDamage { .. } => {
                    let mut damage = context.vars.get_int(&VarName::Damage);
                    damage = match value.calculate(&context, world, resources) {
                        Ok(value) => value,
                        Err(_) => damage,
                    };
                    context.vars.insert(VarName::Damage, Var::Int(damage));
                }
                _ => {}
            },
            Trigger::ModifyOutgoingDamage { value } => match event {
                Event::ModifyOutgoingDamage { .. } => {
                    let mut damage = context.vars.get_int(&VarName::Damage);
                    damage = match value.calculate(&context, world, resources) {
                        Ok(value) => value,
                        Err(_) => damage,
                    };
                    context.vars.insert(VarName::Damage, Var::Int(damage));
                }
                _ => {}
            },
            Trigger::ChangeVarInt { var, delta } => match event {
                Event::ModifyContext { .. } => match delta.calculate(&context, world, resources) {
                    Ok(delta) => context
                        .vars
                        .insert(*var, Var::Int(delta + context.vars.get_int(var))),
                    Err(_) => {}
                },
                _ => {}
            },
            _ => {}
        }
        context
    }
}
