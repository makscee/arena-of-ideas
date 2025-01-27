use super::*;

pub trait ActionsImpl {
    fn process(&self, context: &mut Context) -> Result<Vec<BattleAction>, ExpressionError>;
}

pub trait ActionImpl {
    fn process(&self, context: &mut Context) -> Result<Vec<BattleAction>, ExpressionError>;
}

impl ActionsImpl for Actions {
    fn process(&self, context: &mut Context) -> Result<Vec<BattleAction>, ExpressionError> {
        let mut actions: Vec<BattleAction> = default();
        for a in &self.0 {
            actions.extend(a.process(context)?);
        }
        Ok(actions)
    }
}

impl ActionImpl for Action {
    fn process(&self, context: &mut Context) -> Result<Vec<BattleAction>, ExpressionError> {
        let mut actions = Vec::default();
        match self {
            Action::Noop => {}
            Action::SetValue(x) => {
                context.set_value(x.get_value(context)?);
            }
            Action::AddValue(x) => {
                context.set_value(context.get_value()?.add(&x.get_value(context)?)?);
            }
            Action::SubtractValue(x) => {
                context.set_value(context.get_value()?.sub(&x.get_value(context)?)?);
            }
            Action::SetTarget(x) => {
                context.set_target(x.get_entity(context)?);
            }
            Action::DealDamage => {
                let owner = context.get_owner()?;
                let target = context.get_target()?;
                let value = context.get_value()?.get_i32()?;
                if value > 0 {
                    actions.push(BattleAction::Damage(owner, target, value));
                }
            }
            Action::Repeat(x, vec) => {
                for _ in 0..x.get_i32(context)? {
                    for a in vec {
                        actions.extend(a.process(context)?);
                    }
                }
            }
            Action::MultipleTargets(x, vec) => {
                for target in x.get_entity_list(context)? {
                    context.set_target(target);
                    for a in vec {
                        actions.extend(a.process(context)?);
                    }
                }
            }
        };
        Ok(actions)
    }
}
