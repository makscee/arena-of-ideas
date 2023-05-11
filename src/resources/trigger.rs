use super::*;

#[derive(Debug, Deserialize, Serialize, Clone, strum_macros::AsRefStr)]
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
    ShopStart { effect: EffectWrapped },
    ShopEnd { effect: EffectWrapped },
    TurnStart { effect: EffectWrapped },
    TurnEnd { effect: EffectWrapped },
    OnBuy { effect: EffectWrapped },
    OnSell { effect: EffectWrapped },
    AnyBuy { effect: EffectWrapped },
    AnySell { effect: EffectWrapped },
    BeforeStrike { effect: EffectWrapped },
    AfterStrike { effect: EffectWrapped },
    AddToTeam { effect: EffectWrapped },
    RemoveFromTeam { effect: EffectWrapped },
    ModifyIncomingDamage { value: ExpressionInt },
    ModifyOutgoingDamage { value: ExpressionInt },
    ChangeVarInt { var: VarName, delta: ExpressionInt }, // Preferred for stat changes
    Noop,
    AnyDeath { effect: EffectWrapped },
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
                triggers.iter().enumerate().for_each(|(i, trigger)| {
                    trigger.catch_event(
                        event,
                        action_queue,
                        context.clone_stack_string(&format!("list {i}")),
                        logger,
                    )
                });
            }
            Trigger::OnBuy { .. } => match event {
                Event::Buy { owner } => {
                    if context.owner() == Some(*owner) {
                        self.fire(action_queue, context, logger);
                    }
                }
                _ => {}
            },
            Trigger::OnSell { .. } => match event {
                Event::Sell { owner } => {
                    if context.owner() == Some(*owner) {
                        self.fire(action_queue, context, logger);
                    }
                }
                _ => {}
            },
            Trigger::AnyBuy { .. } => match event {
                Event::Buy { owner } => {
                    if context.owner() != Some(*owner) {
                        self.fire(action_queue, context, logger);
                    }
                }
                _ => {}
            },
            Trigger::AnySell { .. } => match event {
                Event::Sell { owner } => {
                    if context.owner() != Some(*owner) {
                        self.fire(action_queue, context, logger);
                    }
                }
                _ => {}
            },
            Trigger::AnyDeath { .. } => match event {
                Event::UnitDeath { target } => {
                    if context.owner() != Some(*target) {
                        self.fire(action_queue, context, logger);
                    }
                }
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
            Trigger::ShopStart { .. } => match event {
                Event::ShopStart { .. } => self.fire(action_queue, context, logger),
                _ => {}
            },
            Trigger::ShopEnd { .. } => match event {
                Event::ShopEnd { .. } => self.fire(action_queue, context, logger),
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

    fn fire(&self, action_queue: &mut VecDeque<Action>, mut context: Context, logger: &Logger) {
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
            | Trigger::ShopStart { effect }
            | Trigger::ShopEnd { effect }
            | Trigger::TurnStart { effect }
            | Trigger::TurnEnd { effect }
            | Trigger::BeforeStrike { effect }
            | Trigger::AfterStrike { effect }
            | Trigger::OnBuy { effect }
            | Trigger::OnSell { effect }
            | Trigger::AnyBuy { effect }
            | Trigger::AnySell { effect }
            | Trigger::AnyDeath { effect }
            | Trigger::AddToTeam { effect }
            | Trigger::RemoveFromTeam { effect }
            | Trigger::OnStatusAdd { effect }
            | Trigger::OnStatusRemove { effect }
            | Trigger::OnStatusChargeAdd { effect }
            | Trigger::OnStatusChargeRemove { effect } => {
                logger.log(
                    || format!("Caught trigger {:?}, {}", self.as_ref(), context),
                    &LogContext::Trigger,
                );
                context.stack_string(&format!("Trigger {}", self.as_ref()));
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

    pub fn calculate_event(
        &self,
        event: &Event,
        context: &Context,
        world: &legion::World,
        resources: &Resources,
    ) -> Result<Vec<ContextLayer>, Error> {
        let mut extra_layers = Vec::default();
        match self {
            Trigger::List { triggers } => {
                for trigger in triggers {
                    extra_layers.extend(trigger.calculate_event(event, context, world, resources)?);
                }
            }
            Trigger::ModifyIncomingDamage { value } => match event {
                Event::ModifyIncomingDamage { .. } => {
                    let value = value.calculate(&context, world, resources)?;
                    extra_layers.push(ContextLayer::Var {
                        var: VarName::Damage,
                        value: Var::Int(value),
                    });
                }
                _ => {}
            },
            Trigger::ModifyOutgoingDamage { value } => match event {
                Event::ModifyOutgoingDamage { .. } => {
                    let value = value.calculate(&context, world, resources)?;
                    extra_layers.push(ContextLayer::Var {
                        var: VarName::Damage,
                        value: Var::Int(value),
                    });
                }
                _ => {}
            },
            Trigger::ChangeVarInt { var, delta } => match event {
                Event::ModifyContext { .. } => {
                    let value = context
                        .get_int(var, world)
                        .context(format!("Failed to find original var {var}"))?;
                    let delta = delta.calculate(context, world, resources)?;
                    extra_layers.push(ContextLayer::Var {
                        var: *var,
                        value: Var::Int(value + delta),
                    });
                }
                _ => {}
            },
            _ => {}
        };
        Ok(extra_layers)
    }
}
