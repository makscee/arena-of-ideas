use super::*;

#[reducer]
fn sync_assets(
    ctx: &ReducerContext,
    global_settings: GlobalSettings,
    all: Vec<String>,
) -> Result<(), String> {
    GlobalData::init(ctx);
    ctx.is_admin()?;
    global_settings.replace(ctx);
    for n in ctx.db.nodes_world().iter() {
        ctx.db.nodes_world().delete(n);
    }
    let all = All::from_strings(0, &all).to_e_s("Failed to parse All structure")?;
    all.save(ctx);
    Ok(())
}

#[reducer]
fn incubator_update_core(ctx: &ReducerContext) -> Result<(), String> {
    ctx.is_admin()?;
    Ok(())
}
