use anyhow::{anyhow, Result};

use super::*;

#[spacetimedb(table)]
pub struct Tower {
    #[primarykey]
    #[autoinc]
    id: u64,
    owner: Identity,
    creator: Identity,
    defeaters: Vec<Identity>,
    levels: Vec<String>,
    status: TowerStatus,
}

#[derive(SpacetimeType, PartialEq, Eq)]
pub enum TowerStatus {
    Fresh(String),
    Beaten(String),
}

#[spacetimedb(reducer)]
pub fn sync_tower_levels(
    ctx: ReducerContext,
    tower_id: u64,
    levels: Vec<String>,
) -> anyhow::Result<()> {
    let mut tower = Tower::filter_by_id(&tower_id).context("Tower not found")?;
    if tower.owner != ctx.sender {
        return Err(anyhow!("Tried to modified tower not owned by sender"));
    }
    tower.levels = levels;
    Tower::update_by_id(&tower_id, tower);
    Ok(())
}

#[spacetimedb(reducer)]
pub fn finish_building_tower(
    ctx: ReducerContext,
    levels: Vec<String>,
    owner_team: String,
) -> Result<()> {
    Tower::insert(Tower {
        id: 0,
        owner: ctx.sender.clone(),
        creator: ctx.sender.clone(),
        defeaters: Vec::default(),
        levels,
        status: TowerStatus::Fresh(owner_team),
    })?;
    Ok(())
}

#[spacetimedb(reducer)]
pub fn beat_tower(
    ctx: ReducerContext,
    tower_id: u64,
    levels: Vec<String>,
    owner_team: String,
) -> Result<()> {
    let mut tower = Tower::filter_by_id(&tower_id).context("Tower not found")?;
    if !matches!(tower.status, TowerStatus::Beaten(..))
        && !matches!(tower.status, TowerStatus::Fresh(..))
    {
        return Err(anyhow!("Tried to beat tower of wrong status"));
    }
    tower.owner = ctx.sender;
    tower.defeaters.push(ctx.sender);
    tower.levels = levels;
    tower.status = TowerStatus::Beaten(owner_team);
    Tower::update_by_id(&tower_id, tower);
    Ok(())
}
