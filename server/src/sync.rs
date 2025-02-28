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
    for n in ctx.db.tnodes().iter() {
        ctx.db.tnodes().delete(n);
    }
    for r in ctx.db.nodes_relations().iter() {
        ctx.db.nodes_relations().delete(r);
    }
    let all = All::from_strings(0, &all).to_e_s("Failed to parse All structure")?;
    all.save(ctx);
    Ok(())
}
