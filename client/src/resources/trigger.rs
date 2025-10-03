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
            Event::Death(entity) => {
                let entity = entity.to_e();
                let Some(owner) = ctx.owner().and_then(|id| ctx.entity(id).ok()) else {
                    return Ok(false);
                };
                if matches!(self, Trigger::BeforeDeath) && owner == entity {
                    return Ok(true);
                }
            }
            Event::OutgoingDamage(source, _) => {
                let source = source.to_e();
                let Ok(owner) = ctx.owner_entity() else {
                    return Ok(false);
                };
                let owner = ctx
                    .load_first_parent::<NFusion>(ctx.id(owner)?)?
                    .entity(ctx)?;
                if matches!(self, Trigger::ChangeOutgoingDamage) && owner == source {
                    return Ok(true);
                }
            }
            Event::IncomingDamage(_source, _target) => {
                todo!()
            }
        }
        Ok(false)
    }
}
