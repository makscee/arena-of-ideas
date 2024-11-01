use representation::TRepresentation;

use super::*;

#[spacetimedb(table(public))]
pub struct TIncubator {
    #[primarykey]
    id: u64,
    owner: u64,
    #[unique]
    unit: String,
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
fn incubator_post_unit(
    ctx: ReducerContext,
    mut unit: TBaseUnit,
    representation: String,
) -> Result<(), String> {
    let user = ctx.user()?;
    if TBaseUnit::filter_by_name(&unit.name).is_some() {
        return Err(format!("Name {} already taken", unit.name));
    }
    unit.pool = UnitPool::Incubator;
    TIncubator::insert(TIncubator {
        id: next_id(),
        owner: user.id,
        unit: unit.name.clone(),
    })?;
    TRepresentation::insert(TRepresentation {
        id: unit.name.clone(),
        data: representation,
    })?;
    TBaseUnit::insert(unit)?;
    Ok(())
}

#[spacetimedb(reducer)]
fn incubator_update_unit(
    ctx: ReducerContext,
    unit: TBaseUnit,
    representation: String,
) -> Result<(), String> {
    let i = TIncubator::filter_by_unit(&unit.name).context_str("Incubator entry not found")?;
    incubator_delete_unit(ctx, i.id)?;
    incubator_post_unit(ctx, unit, representation)?;
    Ok(())
}

#[spacetimedb(reducer)]
fn incubator_delete_unit(ctx: ReducerContext, id: u64) -> Result<(), String> {
    let user = ctx.user()?;
    let i = TIncubator::filter_by_id(&id).context_str("Incubator entry not found")?;
    if i.owner != user.id {
        return Err(format!(
            "Incubator entry for {} not owned by {}",
            i.unit, user.id
        ));
    }
    let unit = i.unit;
    if let Some(unit) = TBaseUnit::filter_by_name(&unit) {
        if unit.pool != UnitPool::Incubator {
            return Err(format!("Unit {} is not in Incubator pool", unit.name));
        }
        TBaseUnit::delete_by_name(&unit.name);
    } else {
        return Err(format!("Unit {} not found", unit));
    }
    TIncubator::delete_by_unit(&unit);
    TRepresentation::delete_by_id(&unit);
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
