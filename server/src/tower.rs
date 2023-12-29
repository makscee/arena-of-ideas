use anyhow::{anyhow, Result};
use itertools::Itertools;

use crate::user::User;

use super::*;

#[spacetimedb(table)]
pub struct Tower {
    #[primarykey]
    #[autoinc]
    id: u64,
    owner: String,
    creator: String,
    defeaters: Vec<String>,
    levels: Vec<String>,
    status: TowerStatus,
}

#[derive(SpacetimeType, PartialEq, Eq)]
pub enum TowerStatus {
    Fresh(String),
    Beaten(String),
}

#[spacetimedb(reducer)]
fn sync_tower_levels(
    ctx: ReducerContext,
    tower_id: u64,
    levels: Vec<String>,
) -> anyhow::Result<()> {
    let mut tower = Tower::filter_by_id(&tower_id).context("Tower not found")?;
    let user = User::find_by_identity(&ctx.sender).context("User not found")?;
    if tower.owner != user.name {
        return Err(anyhow!("Tried to modified tower not owned by sender"));
    }
    tower.levels = levels;
    Tower::update_by_id(&tower_id, tower);
    Ok(())
}

#[spacetimedb(reducer)]
fn finish_building_tower(
    ctx: ReducerContext,
    levels: Vec<String>,
    owner_team: String,
) -> Result<()> {
    let user = User::find_by_identity(&ctx.sender).context("User not found")?;
    Tower::insert(Tower {
        id: 0,
        owner: user.name.clone(),
        creator: user.name,
        defeaters: Vec::default(),
        levels,
        status: TowerStatus::Fresh(owner_team),
    })?;
    Ok(())
}

#[spacetimedb(reducer)]
fn beat_tower(
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
    let user = User::find_by_identity(&ctx.sender).context("User not found")?;
    tower.owner = user.name.clone();
    tower.defeaters.push(user.name);
    tower.levels = levels;
    tower.status = TowerStatus::Beaten(owner_team);
    Tower::update_by_id(&tower_id, tower);
    Ok(())
}

impl Tower {
    pub fn apply_name_change(old: String, new: String) {
        for mut tower in Tower::iter() {
            let mut updated = false;
            if tower.creator.eq(&old) {
                tower.creator = new.clone();
                updated = true;
            }
            if tower.owner.eq(&old) {
                tower.owner = new.clone();
                updated = true;
            }
            if tower.defeaters.contains(&old) {
                let (pos, _) = tower
                    .defeaters
                    .iter()
                    .find_position(|n| old.eq(*n))
                    .unwrap();
                tower.defeaters[pos] = new.clone();
                updated = true;
            }
            if updated {
                Tower::update_by_id(&tower.id.clone(), tower);
            }
        }
    }
}
