use super::*;

#[spacetimedb(table(public))]
pub struct TIncubator {
    #[primarykey]
    id: u64,
    owner: u64,
    unit: Vec<TBaseUnit>,
}

#[spacetimedb(table(public))]
pub struct TIncubatorVote {
    #[primarykey]
    id: u64,
    owner: u64,
    target: u64,
    vote: bool,
}

#[spacetimedb(table(public))]
pub struct TIncubatorFavorite {
    #[primarykey]
    owner: u64,
    target: u64,
}

#[spacetimedb(reducer)]
fn incubator_post(ctx: ReducerContext, unit: TBaseUnit) -> Result<(), String> {
    let user = ctx.user()?;
    if TBaseUnit::filter_by_name(&unit.name).is_some() {
        return Err(format!("Name {} already taken", unit.name));
    }
    TIncubator::insert(TIncubator {
        id: next_id(),
        owner: user.id,
        unit: vec![unit],
    })?;
    Ok(())
}

#[spacetimedb(reducer)]
fn incubator_update(ctx: ReducerContext, id: u64, unit: TBaseUnit) -> Result<(), String> {
    let user = ctx.user()?;
    let mut i = TIncubator::filter_by_id(&id).context_str("Incubator entry not found")?;
    if i.owner != user.id {
        return Err(format!(
            "Incubator entry for {} not owned by {}",
            i.unit.last().unwrap().name,
            user.id
        ));
    }
    i.unit.push(unit);
    while i.unit.len() > 15 {
        i.unit.remove(0);
    }
    TIncubator::update_by_id(&id, i);
    Ok(())
}

#[spacetimedb(reducer)]
fn incubator_delete(ctx: ReducerContext, id: u64) -> Result<(), String> {
    let user = ctx.user()?;
    let i = TIncubator::filter_by_id(&id).context_str("Incubator entry not found")?;
    if i.owner != user.id {
        return Err(format!(
            "Incubator entry for {} not owned by {}",
            i.unit.last().unwrap().name,
            user.id
        ));
    }
    TIncubator::delete_by_id(&id);
    TIncubatorVote::delete_by_target(&id);
    TIncubatorFavorite::delete_by_target(&id);
    Ok(())
}

#[spacetimedb(reducer)]
fn incubator_vote(ctx: ReducerContext, id: u64, vote: bool) -> Result<(), String> {
    let user = ctx.user()?;
    if let Some(mut i) = TIncubatorVote::filter_by_owner(&user.id).find(|d| d.target == id) {
        i.vote = vote;
        TIncubatorVote::update_by_id(&i.id.clone(), i);
    } else {
        TIncubatorVote::insert(TIncubatorVote {
            id: next_id(),
            owner: user.id,
            target: id,
            vote,
        })?;
    }
    Ok(())
}

#[spacetimedb(reducer)]
fn incubator_favorite(ctx: ReducerContext, id: u64) -> Result<(), String> {
    let user = ctx.user()?;
    TIncubatorFavorite::delete_by_owner(&user.id);
    TIncubatorFavorite::insert(TIncubatorFavorite {
        owner: user.id,
        target: id,
    })?;
    Ok(())
}
