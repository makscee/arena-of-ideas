use super::*;

pub trait ReactionImpl {
    fn react(
        &self,
        event: &Event,
        context: &mut Context,
    ) -> Result<Vec<BattleAction>, ExpressionError>;
}

impl ReactionImpl for Reaction {
    fn react(
        &self,
        event: &Event,
        context: &mut Context,
    ) -> Result<Vec<BattleAction>, ExpressionError> {
        match event {
            Event::BattleStart => {
                if matches!(&self.trigger, Trigger::BattleStart) {
                    return self.action.process(context);
                }
            }
            Event::TurnEnd => {
                if matches!(&self.trigger, Trigger::TurnEnd) {
                    return self.action.process(context);
                }
            }
            Event::UpdateStat(e_var) => {
                if matches!(&self.trigger, Trigger::ChangeStats(t_var) if e_var == t_var) {
                    return self.action.process(context);
                }
            }
        }
        Ok(default())
    }
}
