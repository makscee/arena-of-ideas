use crate::unit::TableUnit;

use super::*;

#[spacetimedb(table)]
pub struct ArenaPool {
    #[primarykey]
    #[autoinc]
    pub id: u64,
    pub owner: u64,
    pub round: u8,
    pub team: Vec<TableUnit>,
}

#[spacetimedb(reducer)]
fn upload_pool(ctx: ReducerContext, pool: Vec<ArenaPool>) -> Result<(), String> {
    UserRight::UnitSync.check(&ctx.sender)?;
    for team in pool {
        ArenaPool::delete_by_id(&team.id);
        ArenaPool::insert(team)?;
    }
    Ok(())
}
