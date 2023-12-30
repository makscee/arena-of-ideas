use crate::user_access::UserRight;

use super::*;

#[spacetimedb(table)]
pub struct Ability {
    #[primarykey]
    pub name: String,
    pub data: String,
}

#[spacetimedb(reducer)]
fn sync_abilities(ctx: ReducerContext, abilities: Vec<Ability>) -> Result<(), String> {
    UserRight::UnitSync.check(&ctx.sender)?;
    for ability in Ability::iter() {
        Ability::delete_by_name(&ability.name);
    }
    for ability in abilities {
        Ability::insert(ability)?;
    }
    Ok(())
}
