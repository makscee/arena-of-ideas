use crate::user_access::UserRight;

use super::*;

#[spacetimedb(table)]
pub struct Summon {
    #[primarykey]
    pub name: String,
    pub data: String,
}

#[spacetimedb(reducer)]
fn sync_summons(ctx: ReducerContext, summons: Vec<Summon>) -> Result<(), String> {
    UserRight::UnitSync.check(&ctx.sender)?;
    for summon in Summon::iter() {
        Summon::delete_by_name(&summon.name);
    }
    for summon in summons {
        Summon::insert(summon)?;
    }
    Ok(())
}
