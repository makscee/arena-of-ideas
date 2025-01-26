use super::*;

#[derive(Component)]
pub struct Reaction {
    pub trigger: Trigger,
    pub action: Action,
}

impl Reaction {
    pub fn react(
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
