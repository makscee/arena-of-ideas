use crate::user_access::UserRight;

use super::*;

#[spacetimedb(table)]
pub struct Unit {
    #[primarykey]
    pub name: String,
    pub data: String,
    pub pool: UnitPool,
}

#[derive(SpacetimeType)]
pub enum UnitPool {
    Hero,
    Enemy,
}

#[spacetimedb(reducer)]
fn sync_units(ctx: ReducerContext, units: Vec<Unit>) -> Result<(), String> {
    UserRight::UnitSync.check(&ctx.sender)?;
    for unit in Unit::iter() {
        Unit::delete_by_name(&unit.name);
    }
    for unit in units {
        Unit::insert(unit)?;
    }
    Ok(())
}
