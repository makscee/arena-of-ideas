use super::*;

#[spacetimedb(table)]
#[derive(Clone)]
pub struct TableUnit {
    #[primarykey]
    pub name: String,
    pub hp: i32,
    pub atk: i32,
    pub house: String,
    pub description: String,
    pub stacks: i32,
    pub level: i32,
    pub statuses: Vec<StatusCharges>,
    pub trigger: String,
    pub representation: String,
    pub state: String,
}

#[derive(SpacetimeType, Clone)]
pub struct StatusCharges {
    pub name: String,
    pub charges: i32,
}

#[spacetimedb(reducer)]
fn sync_units(ctx: ReducerContext, units: Vec<TableUnit>) -> Result<(), String> {
    UserRight::UnitSync.check(&ctx.sender)?;
    for unit in TableUnit::iter() {
        TableUnit::delete_by_name(&unit.name);
    }
    for unit in units {
        TableUnit::insert(unit)?;
    }
    Ok(())
}
