use super::*;

#[reducer]
fn sync_assets(ctx: &ReducerContext, global_settings: GlobalSettings) -> Result<(), String> {
    ctx.is_admin()?;
    global_settings.replace(ctx);
    Ok(())
}
