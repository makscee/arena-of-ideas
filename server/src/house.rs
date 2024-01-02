use super::*;

#[spacetimedb(table)]
pub struct House {
    #[primarykey]
    pub name: String,
    pub data: String,
}

#[spacetimedb(reducer)]
fn sync_houses(ctx: ReducerContext, houses: Vec<House>) -> Result<(), String> {
    UserRight::UnitSync.check(&ctx.sender)?;
    for house in House::iter() {
        House::delete_by_name(&house.name);
    }
    for house in houses {
        House::insert(house)?;
    }
    Ok(())
}
