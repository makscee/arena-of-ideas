use super::*;

#[reducer]
fn sync_assets(
    ctx: &ReducerContext,
    global_settings: GlobalSettings,
    houses: Vec<Vec<String>>,
) -> Result<(), String> {
    ctx.is_admin()?;
    global_settings.replace(ctx);
    for n in ctx.db.tnodes().iter() {
        ctx.db.tnodes().delete(n);
    }
    for house in houses {
        let house = House::from_strings(0, &house).to_e_s("Failed to parse House")?;
        // house.to_table(ctx, 0);
    }
    Ok(())
}
