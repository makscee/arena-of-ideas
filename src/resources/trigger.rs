use super::*;

pub trait TriggerImpl {
    fn fire(&self, event: &Event, context: &Context) -> bool;
}

impl TriggerImpl for Trigger {
    fn fire(&self, event: &Event, context: &Context) -> bool {
        match event {
            Event::BattleStart => {
                if matches!(self, Trigger::BattleStart) {
                    return true;
                }
            }
            Event::TurnEnd => {
                if matches!(self, Trigger::TurnEnd) {
                    return true;
                }
            }
            Event::UpdateStat(e_var) => {
                if matches!(self, Trigger::ChangeStat(t_var) if e_var == t_var) {
                    return true;
                }
            }
            Event::Death(entity) => {
                let entity = entity.to_e();
                let Ok(owner) = context.owner_entity() else {
                    return false;
                };
                if matches!(self, Trigger::BeforeDeath) && owner == entity {
                    return true;
                }
            }
        }
        false
    }
}
