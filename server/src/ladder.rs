use anyhow::{anyhow, Result};

use super::*;

#[spacetimedb(table)]
pub struct Ladder {
    #[primarykey]
    #[autoinc]
    id: u64,
    owner: Identity,
    creator: Identity,
    defeaters: Vec<Identity>,
    levels: Vec<String>,
    status: LadderStatus,
}

#[derive(SpacetimeType, PartialEq, Eq)]
pub enum LadderStatus {
    Building,
    Fresh(String),
    Beaten(String),
}

#[spacetimedb(reducer)]
pub fn start_new_ladder(ctx: ReducerContext) -> Result<(), String> {
    for ladder in Ladder::filter_by_owner(&ctx.sender) {
        if ladder.status.eq(&LadderStatus::Building) {
            Ladder::delete_by_id(&ladder.id);
        }
    }
    Ladder::insert(Ladder {
        id: 0,
        owner: ctx.sender.clone(),
        creator: ctx.sender.clone(),
        defeaters: Vec::default(),
        levels: Vec::default(),
        status: LadderStatus::Building,
    })?;
    Ok(())
}

#[spacetimedb(reducer)]
pub fn add_ladder_levels(
    ctx: ReducerContext,
    ladder_id: u64,
    mut levels: Vec<String>,
) -> anyhow::Result<()> {
    let mut ladder = Ladder::filter_by_id(&ladder_id).context("Ladder not found")?;
    if ladder.owner != ctx.sender {
        return Err(anyhow!("Tried to modified ladder not owned by sender"));
    }
    ladder.levels.append(&mut levels);
    Ladder::update_by_id(&ladder_id, ladder);
    Ok(())
}

#[spacetimedb(reducer)]
pub fn finish_building_ladder(ctx: ReducerContext, owner_team: String) -> Result<()> {
    let mut ladder = Ladder::filter_by_creator(&ctx.sender)
        .find(|l| l.status.eq(&LadderStatus::Building))
        .context("No building ladder found")?;
    ladder.status = LadderStatus::Fresh(owner_team);
    Ladder::update_by_id(&ladder.id.clone(), ladder);
    Ok(())
}

#[spacetimedb(reducer)]
pub fn beat_ladder(
    ctx: ReducerContext,
    ladder_id: u64,
    level: String,
    owner_team: String,
) -> Result<()> {
    let mut ladder = Ladder::filter_by_id(&ladder_id).context("Ladder not found")?;
    if !matches!(ladder.status, LadderStatus::Beaten(..))
        && !matches!(ladder.status, LadderStatus::Fresh(..))
    {
        return Err(anyhow!("Tried to beat ladder of wrong status"));
    }
    ladder.owner = ctx.sender;
    ladder.defeaters.push(ctx.sender);
    ladder.levels.push(level);
    ladder.status = LadderStatus::Beaten(owner_team);
    Ladder::update_by_id(&ladder_id, ladder);
    Ok(())
}
