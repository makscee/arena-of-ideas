use super::*;

#[spacetimedb::table(public, name = team)]
#[derive(Clone)]
pub struct TTeam {
    #[primary_key]
    pub id: u64,
    pub name: String,
    #[index(btree)]
    pub owner: u64,
    pub units: Vec<FusedUnit>,
    pub pool: TeamPool,
}

#[derive(SpacetimeType, Clone, Copy)]
pub enum TeamPool {
    Owned,
    Arena,
    Enemy,
}

impl TTeam {
    pub fn get(ctx: &ReducerContext, id: u64) -> Result<Self, String> {
        ctx.db.team().id().find(id).context_str("Team not found")
    }
    pub fn get_owned(ctx: &ReducerContext, team_id: u64, owner_id: u64) -> Result<Self, String> {
        let team = TTeam::get(ctx, team_id)?;
        if team.owner != owner_id {
            return Err(format!("Team#{} not owned by player#{}", team.id, owner_id));
        }
        Ok(team)
    }
    #[must_use]
    pub fn new(ctx: &ReducerContext, owner: u64, pool: TeamPool) -> Self {
        Self {
            id: next_id(ctx),
            name: String::new(),
            owner,
            units: Vec::new(),
            pool,
        }
    }
    #[must_use]
    pub fn name(mut self, name: String) -> Self {
        self.name = name;
        self
    }
    #[must_use]
    pub fn units(mut self, units: Vec<FusedUnit>) -> Self {
        self.units = units;
        self
    }
    #[must_use]
    pub fn apply_limit(mut self, ctx: &ReducerContext) -> Self {
        let max_len = GlobalSettings::get(ctx).arena.team_slots as usize;
        if self.units.len() <= max_len {
            return self;
        }
        let _ = self.units.split_off(max_len);
        self
    }
    pub fn apply_empty_stat_bonus(mut self, ctx: &ReducerContext) -> Self {
        let bonus = GlobalSettings::get(ctx).arena.team_slots as i32 - self.units.len() as i32;
        for unit in self.units.iter_mut() {
            unit.pwr += bonus;
            unit.hp += bonus;
            unit.pwr_mutation += bonus;
            unit.hp_mutation += bonus;
        }
        self
    }
    pub fn save(self, ctx: &ReducerContext) -> u64 {
        ctx.db.team().id().delete(self.id);
        ctx.db.team().insert(self).id
    }
    pub fn save_clone(&self, ctx: &ReducerContext) -> Self {
        let mut c = self.clone();
        c.id = next_id(ctx);
        ctx.db.team().insert(c)
    }
    pub fn get_unit(&self, i: u8) -> Result<&FusedUnit, String> {
        self.units
            .get(i as usize)
            .with_context_str(|| format!("Failed to find unit team#{} slot {i}", self.id))
    }
}

#[spacetimedb::reducer]
fn team_create(ctx: &ReducerContext, name: String) -> Result<(), String> {
    if name.len() > 20 {
        return Err("Name is too long (max 20 chars)".into());
    }
    if name.is_empty() {
        return Err("Name can't be empty".into());
    }
    let player = ctx.player()?;
    TWallet::change(ctx, player.id, -GlobalSettings::get(ctx).create_team_cost)?;
    TTeam::new(ctx, player.id, TeamPool::Owned)
        .name(name)
        .save(ctx);
    Ok(())
}
#[spacetimedb::reducer]
fn team_add_unit(ctx: &ReducerContext, team: u64, unit: u64) -> Result<(), String> {
    let player = ctx.player()?;
    let mut team = TTeam::get_owned(ctx, team, player.id)?;
    let unit = ctx
        .db
        .unit_item()
        .owner()
        .filter(player.id)
        .find(|u| u.unit.id == unit)
        .context_str("Unit not found")?;
    ctx.db.unit_item().id().delete(unit.id);
    team.units.push(unit.unit);
    team.save(ctx);
    Ok(())
}
#[spacetimedb::reducer]
fn team_remove_unit(ctx: &ReducerContext, team: u64, unit: u64) -> Result<(), String> {
    let player = ctx.player()?;
    let mut team = TTeam::get_owned(ctx, team, player.id)?;
    if let Some(pos) = team.units.iter().position(|u| u.id == unit) {
        let unit = team.units.remove(pos);
        ctx.db.unit_item().insert(TUnitItem {
            id: next_id(ctx),
            owner: player.id,
            unit,
        });
        team.save(ctx);
    } else {
        return Err(format!("Unit#{} not found", unit));
    }
    Ok(())
}
#[spacetimedb::reducer]
fn team_swap_units(ctx: &ReducerContext, team: u64, from: u8, to: u8) -> Result<(), String> {
    let player = ctx.player()?;
    let mut team = TTeam::get_owned(ctx, team, player.id)?;
    let from = from as usize;
    let to = (to as usize).min(team.units.len() - 1);
    if from >= team.units.len() {
        return Err("Wrong from index".into());
    }
    let unit = team.units.remove(from);
    team.units.insert(to, unit);
    team.save(ctx);
    Ok(())
}
#[spacetimedb::reducer]
fn team_disband(ctx: &ReducerContext, team: u64) -> Result<(), String> {
    let player = ctx.player()?;
    let mut team = TTeam::get_owned(ctx, team, player.id)?;
    for unit in team.units.drain(..) {
        ctx.db.unit_item().insert(TUnitItem {
            id: next_id(ctx),
            owner: player.id,
            unit,
        });
    }
    ctx.db.team().delete(team);
    Ok(())
}
#[spacetimedb::reducer]
fn team_rename(ctx: &ReducerContext, team: u64, name: String) -> Result<(), String> {
    let player = ctx.player()?;
    let mut team = TTeam::get_owned(ctx, team, player.id)?;
    if name.is_empty() {
        return Err("Name can't be empty".into());
    }
    if name.len() > 20 {
        return Err("Name is too long (max 20 chars)".into());
    }
    team.name = name;
    team.save(ctx);
    Ok(())
}
