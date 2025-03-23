use super::*;

#[reducer]
fn sync_assets(
    ctx: &ReducerContext,
    global_settings: GlobalSettings,
    all: Vec<TNode>,
) -> Result<(), String> {
    GlobalData::init(ctx);
    ctx.is_admin()?;
    global_settings.replace(ctx);
    for n in ctx.db.nodes_world().iter() {
        ctx.db.nodes_world().delete(n);
    }
    let all = All::from_tnodes(0, &all).to_e_s("Failed to parse All structure")?;
    all.save(ctx);
    Ok(())
}

#[reducer]
fn incubator_merge(ctx: &ReducerContext) -> Result<(), String> {
    ctx.is_admin()?;
    for house in All::load(ctx).core_load(ctx)? {
        house.delete_recursive(ctx);
    }
    for house in House::collect_children_of_id(ctx, ID_INCUBATOR) {
        house.fill_from_incubator(ctx).clone(ctx, ID_ALL);
    }
    Ok(())
}
