use super::*;

#[spacetimedb::table(public, name = unit_balance)]
pub struct TUnitBalance {
    #[primary_key]
    pub id: u64,
    #[index(btree)]
    pub owner: u64,
    pub unit: String,
    pub vote: i32,
}

#[spacetimedb::reducer]
fn unit_balance_vote(ctx: &ReducerContext, unit: String, vote: i32) -> Result<(), String> {
    let player = ctx.player()?;
    let already_voted = ctx
        .db
        .unit_balance()
        .owner()
        .filter(player.id)
        .find(|u| u.unit.eq(&unit));
    if let Some(mut row) = already_voted {
        row.vote = vote;
        ctx.db.unit_balance().id().update(row);
    } else {
        TWallet::change(
            ctx,
            player.id,
            GlobalSettings::get(ctx).meta.balance_vote_reward,
        )?;
        ctx.db.unit_balance().insert(TUnitBalance {
            id: next_id(ctx),
            owner: player.id,
            unit,
            vote,
        });
    }
    Ok(())
}
