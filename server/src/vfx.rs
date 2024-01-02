use super::*;

#[spacetimedb(table)]
pub struct Vfx {
    #[primarykey]
    pub name: String,
    pub data: String,
}

#[spacetimedb(reducer)]
fn sync_vfxs(ctx: ReducerContext, vfxs: Vec<Vfx>) -> Result<(), String> {
    UserRight::UnitSync.check(&ctx.sender)?;
    for vfx in Vfx::iter() {
        Vfx::delete_by_name(&vfx.name);
    }
    for vfx in vfxs {
        Vfx::insert(vfx)?;
    }
    Ok(())
}
