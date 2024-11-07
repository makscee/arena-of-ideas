use base_unit::base_unit;

use super::*;

#[spacetimedb::table(public, name = incubator)]
pub struct TIncubator {
    #[primary_key]
    id: u64,
    owner: u64,
    unit: Vec<TBaseUnit>,
}

#[spacetimedb::table(public, name = incubator_vote)]
pub struct TIncubatorVote {
    #[primary_key]
    id: u64,
    #[index(btree)]
    owner: u64,
    #[index(btree)]
    target: u64,
    vote: bool,
}

#[spacetimedb::table(public, name = incubator_favorite)]
pub struct TIncubatorFavorite {
    #[primary_key]
    owner: u64,
    #[index(btree)]
    target: u64,
}

#[spacetimedb::reducer]
fn incubator_post(ctx: &ReducerContext, unit: TBaseUnit) -> Result<(), String> {
    let player = ctx.player()?;
    if ctx.db.base_unit().name().find(&unit.name).is_some() {
        return Err(format!("Name {} already taken", unit.name));
    }
    ctx.db.incubator().insert(TIncubator {
        id: next_id(ctx),
        owner: player.id,
        unit: vec![unit],
    });
    Ok(())
}

#[spacetimedb::reducer]
fn incubator_update(ctx: &ReducerContext, id: u64, unit: TBaseUnit) -> Result<(), String> {
    let player = ctx.player()?;
    let mut i = ctx
        .db
        .incubator()
        .id()
        .find(id)
        .context_str("Incubator entry not found")?;
    if i.owner != player.id {
        return Err(format!(
            "Incubator entry for {} not owned by {}",
            i.unit.last().unwrap().name,
            player.id
        ));
    }
    i.unit.push(unit);
    while i.unit.len() > 15 {
        i.unit.remove(0);
    }
    ctx.db.incubator().id().update(i);
    Ok(())
}

#[spacetimedb::reducer]
fn incubator_delete(ctx: &ReducerContext, id: u64) -> Result<(), String> {
    let player = ctx.player()?;
    let i = ctx
        .db
        .incubator()
        .id()
        .find(id)
        .context_str("Incubator entry not found")?;
    if i.owner != player.id {
        return Err(format!(
            "Incubator entry for {} not owned by {}",
            i.unit.last().unwrap().name,
            player.id
        ));
    }
    ctx.db.incubator().id().delete(id);
    ctx.db.incubator_vote().target().delete(id);
    ctx.db.incubator_favorite().target().delete(id);
    Ok(())
}

#[spacetimedb::reducer]
fn incubator_vote_set(ctx: &ReducerContext, id: u64, vote: bool) -> Result<(), String> {
    let player = ctx.player()?;

    if let Some(mut i) = ctx
        .db
        .incubator_vote()
        .owner()
        .filter(player.id)
        .find(|d| d.target == id)
    {
        i.vote = vote;
        ctx.db.incubator_vote().id().update(i);
    } else {
        ctx.db.incubator_vote().insert(TIncubatorVote {
            id: next_id(ctx),
            owner: player.id,
            target: id,
            vote,
        });
    }
    Ok(())
}

#[spacetimedb::reducer]
fn incubator_favorite_set(ctx: &ReducerContext, id: u64) -> Result<(), String> {
    let player = ctx.player()?;
    ctx.db.incubator_favorite().owner().delete(player.id);
    ctx.db.incubator_favorite().insert(TIncubatorFavorite {
        owner: player.id,
        target: id,
    });
    Ok(())
}
