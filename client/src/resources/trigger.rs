use super::*;

pub trait TriggerImpl {
    fn fire(&self, event: &Event, context: &ClientContext) -> NodeResult<bool>;
}

impl TriggerImpl for Trigger {
    fn fire(&self, event: &Event, ctx: &ClientContext) -> NodeResult<bool> {
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
            Event::Death(id) => {
                let Some(owner) = ctx.owner() else {
                    return Ok(false);
                };
                if matches!(self, Trigger::BeforeDeath) && owner == *id {
                    return Ok(true);
                }
            }
            Event::OutgoingDamage(source, _) => {
                let Some(owner) = ctx.owner() else {
                    return Ok(false);
                };
                let owner = ctx.load::<NFusion>(owner)?;
                if matches!(self, Trigger::ChangeOutgoingDamage) && owner.id == *source {
                    return Ok(true);
                }
            }
            Event::IncomingDamage(_, target) => {
                let Some(owner) = ctx.owner() else {
                    return Ok(false);
                };
                let owner = ctx.load::<NFusion>(owner)?;
                if matches!(self, Trigger::ChangeIncomingDamage) && owner.id == *target {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }
}
