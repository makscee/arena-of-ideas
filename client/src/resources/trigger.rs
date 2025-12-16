use super::*;

pub trait TriggerImpl {
    fn fire(&self, event: &Event, ctx: &ClientContext) -> NodeResult<bool>;
}

fn get_owner_unit<'a>(ctx: &'a ClientContext) -> NodeResult<Option<&'a NUnit>> {
    let Ok(owner) = ctx.owner() else {
        return Ok(None);
    };
    let owner = ctx
        .load_or_first_parent_recursive_ref::<NUnit>(owner)
        .track()?;
    Ok(Some(owner))
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
                let Some(owner) = get_owner_unit(ctx)? else {
                    return Ok(false);
                };
                if matches!(self, Trigger::BeforeDeath) && owner.id == *id {
                    return Ok(true);
                }
                if matches!(self, Trigger::AllyDeath)
                    && ctx.battle()?.all_allies(owner.id)?.contains(id)
                {
                    return Ok(true);
                }
            }
            Event::OutgoingDamage(source, _) => {
                let Some(owner) = get_owner_unit(ctx)? else {
                    return Ok(false);
                };
                if matches!(self, Trigger::ChangeOutgoingDamage) && owner.id == *source {
                    return Ok(true);
                }
            }
            Event::IncomingDamage(_, target) => {
                let Some(owner) = get_owner_unit(ctx)? else {
                    return Ok(false);
                };
                if matches!(self, Trigger::ChangeIncomingDamage) && owner.id == *target {
                    return Ok(true);
                }
            }
            Event::BeforeStrike(source, _) => {
                let Some(owner) = get_owner_unit(ctx)? else {
                    return Ok(false);
                };
                if matches!(self, Trigger::BeforeStrike) && owner.id == *source {
                    return Ok(true);
                }
            }
            Event::AfterStrike(source, _) => {
                let Some(owner) = get_owner_unit(ctx)? else {
                    return Ok(false);
                };
                if matches!(self, Trigger::AfterStrike) && owner.id == *source {
                    return Ok(true);
                }
            }
            Event::DamageDealt(source, target, _) => {
                let Some(owner) = get_owner_unit(ctx)? else {
                    return Ok(false);
                };
                if matches!(self, Trigger::DamageDealt) && owner.id == *source {
                    return Ok(true);
                }
                if matches!(self, Trigger::DamageTaken) && owner.id == *target {
                    return Ok(true);
                }
            }
            Event::StatusApplied(caster, target, _) => {
                let Some(owner) = get_owner_unit(ctx)? else {
                    return Ok(false);
                };
                if matches!(self, Trigger::StatusGained) && owner.id == *target {
                    return Ok(true);
                }
                if matches!(self, Trigger::StatusApplied) && owner.id == *caster {
                    return Ok(true);
                }
            }
            Event::StatusGained(caster, target) => {
                let Some(owner) = get_owner_unit(ctx)? else {
                    return Ok(false);
                };
                if matches!(self, Trigger::StatusGained) && owner.id == *target {
                    return Ok(true);
                }
            }
            Event::ChangeOutgoingDamage(source, _) => {
                let Some(owner) = get_owner_unit(ctx)? else {
                    return Ok(false);
                };
                if matches!(self, Trigger::ChangeOutgoingDamage) && owner.id == *source {
                    return Ok(true);
                }
            }
            Event::ChangeIncomingDamage(_, target) => {
                let Some(owner) = get_owner_unit(ctx)? else {
                    return Ok(false);
                };
                if matches!(self, Trigger::ChangeIncomingDamage) && owner.id == *target {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }
}
