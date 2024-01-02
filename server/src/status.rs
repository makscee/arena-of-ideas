use super::*;

#[spacetimedb(table)]
pub struct Statuses {
    #[primarykey]
    pub name: String,
    pub data: String,
}

#[spacetimedb(reducer)]
fn sync_statuses(ctx: ReducerContext, statuses: Vec<Statuses>) -> Result<(), String> {
    UserRight::UnitSync.check(&ctx.sender)?;
    for status in Statuses::iter() {
        Statuses::delete_by_name(&status.name);
    }
    for status in statuses {
        Statuses::insert(status)?;
    }
    Ok(())
}
