use super::*;

pub trait ReactionImpl {
    fn react(&self, event: &Event, context: &Context) -> Result<bool, ExpressionError>;
}

impl ReactionImpl for Reaction {
    fn react(&self, event: &Event, context: &Context) -> Result<bool, ExpressionError> {
        match event {
            Event::BattleStart => {
                if matches!(&self.trigger, Trigger::BattleStart) {
                    return Ok(true);
                }
            }
            Event::TurnEnd => {
                if matches!(&self.trigger, Trigger::TurnEnd) {
                    return Ok(true);
                }
            }
            Event::UpdateStat(e_var) => {
                if matches!(&self.trigger, Trigger::ChangeStats(t_var) if e_var == t_var) {
                    return Ok(true);
                }
            }
            Event::Death(entity) => {
                let entity = entity.to_e();
                if matches!(&self.trigger, Trigger::BeforeDeath) && context.get_owner()? == entity {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }
}
