use super::*;

#[reducer]
fn sync_assets(
    ctx: &ReducerContext,
    global_settings: GlobalSettings,
    nodes: String,
) -> Result<(), String> {
    GlobalData::init(ctx);
    ctx.is_admin()?;
    global_settings.replace(ctx);

    Ok(())
}

#[reducer]
fn update_links(ctx: &ReducerContext) -> Result<(), String> {
    ctx.is_admin()?;
    Ok(())
}
