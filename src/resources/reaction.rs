use super::*;

pub trait ReactionImpl {
    fn react(&self, event: &Event) -> bool;
}

impl ReactionImpl for Reaction {
    fn react(&self, event: &Event) -> bool {
        match event {
            Event::BattleStart => {
                if matches!(&self.trigger, Trigger::BattleStart) {
                    return true;
                }
            }
            Event::TurnEnd => {
                if matches!(&self.trigger, Trigger::TurnEnd) {
                    return true;
                }
            }
            Event::UpdateStat(e_var) => {
                if matches!(&self.trigger, Trigger::ChangeStats(t_var) if e_var == t_var) {
                    return true;
                }
            }
        }
        false
    }
}
