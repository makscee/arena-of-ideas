use super::*;

pub trait TriggerImpl {
    fn fire(&self, event: &Event, context: &Context) -> Result<bool, ExpressionError>;
}

impl TriggerImpl for Trigger {
    fn fire(&self, event: &Event, context: &Context) -> Result<bool, ExpressionError> {
        match event {
            Event::BattleStart => {
                if matches!(self, Trigger::BattleStart) {
                    return Ok(true);
                }
            }
            Event::TurnEnd => {
                if matches!(self, Trigger::TurnEnd) {
                    return Ok(true);
                }
            }
            Event::UpdateStat(e_var) => {
                if matches!(self, Trigger::ChangeStat(t_var) if e_var == t_var) {
                    return Ok(true);
                }
            }
            Event::Death(entity) => {
                let entity = entity.to_e();
                let Ok(owner) = context.owner_entity() else {
                    return Ok(false);
                };
                if matches!(self, Trigger::BeforeDeath) && owner == entity {
                    return Ok(true);
                }
            }
            Event::OutgoingDamage(source, _) => {
                let source = source.to_e();
                let Ok(owner) = context.owner_entity() else {
                    return Ok(false);
                };
                let owner = context
                    .first_parent::<NFusion>(context.id(owner)?)?
                    .entity();
                if matches!(self, Trigger::ChangeOutgoingDamage) && owner == source {
                    return Ok(true);
                }
            }
            Event::IncomingDamage(source, target) => {
                todo!()
            }
        }
        Ok(false)
    }
}
