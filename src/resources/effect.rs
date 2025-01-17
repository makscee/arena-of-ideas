use super::*;

pub trait EffectImpl {
    fn process(&self, context: &Context) -> Result<Vec<BattleAction>, ExpressionError>;
}

impl EffectImpl for Effect {
    fn process(&self, context: &Context) -> Result<Vec<BattleAction>, ExpressionError> {
        let mut actions = Vec::default();
        match self {
            Effect::Noop => {}
            Effect::Damage => {
                actions.push(BattleAction::Damage(
                    context.get_owner()?,
                    context.get_target()?,
                    context.get_var(VarName::pwr)?.get_i32()?,
                ));
                actions.push(BattleAction::Wait(0.1));
            }
            Effect::ChangeStatus => {
                let charges = context
                    .get_var(VarName::charges)
                    .and_then(|v| v.get_i32())
                    .unwrap_or(1);
                actions.push(BattleAction::ApplyStatus(context.get_target()?));
            }
        }
        Ok(actions)
    }
}
