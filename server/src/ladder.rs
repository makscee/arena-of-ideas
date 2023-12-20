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
    Fresh(String),
    Beaten(String),
}

#[spacetimedb(reducer)]
pub fn sync_ladder_levels(
    ctx: ReducerContext,
    ladder_id: u64,
    levels: Vec<String>,
) -> anyhow::Result<()> {
    let mut ladder = Ladder::filter_by_id(&ladder_id).context("Ladder not found")?;
    if ladder.owner != ctx.sender {
        return Err(anyhow!("Tried to modified ladder not owned by sender"));
    }
    ladder.levels = levels;
    Ladder::update_by_id(&ladder_id, ladder);
    Ok(())
}

#[spacetimedb(reducer)]
pub fn finish_building_ladder(
    ctx: ReducerContext,
    levels: Vec<String>,
    owner_team: String,
) -> Result<()> {
    Ladder::insert(Ladder {
        id: 0,
        owner: ctx.sender.clone(),
        creator: ctx.sender.clone(),
        defeaters: Vec::default(),
        levels,
        status: LadderStatus::Fresh(owner_team),
    })?;
    Ok(())
}

#[spacetimedb(reducer)]
pub fn beat_ladder(
    ctx: ReducerContext,
    ladder_id: u64,
    levels: Vec<String>,
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
    ladder.levels = levels;
    ladder.status = LadderStatus::Beaten(owner_team);
    Ladder::update_by_id(&ladder_id, ladder);
    Ok(())
}
