use super::*;

#[spacetimedb(table(public))]
pub struct TUnitBalance {
    #[primarykey]
    pub id: u64,
    pub owner: u64,
    pub unit: String,
    pub vote: i32,
}

#[spacetimedb(reducer)]
fn unit_balance_vote(ctx: ReducerContext, unit: String, vote: i32) -> Result<(), String> {
    let user = ctx.user()?;
    let already_voted = TUnitBalance::filter_by_owner(&user.id).find(|u| u.unit.eq(&unit));
    if let Some(mut row) = already_voted {
        row.vote = vote;
        TUnitBalance::update_by_id(&row.id.clone(), row);
    } else {
        TWallet::change(user.id, GlobalSettings::get().meta.balance_vote_reward)?;
        TUnitBalance::insert(TUnitBalance {
            id: next_id(),
            owner: user.id,
            unit,
            vote,
        })?;
    }
    Ok(())
}
