use super::*;

#[reducer]
fn sync_assets(
    ctx: &ReducerContext,
    global_settings: GlobalSettings,
    core: Vec<TNode>,
) -> Result<(), String> {
    GlobalData::init(ctx);
    ctx.is_admin()?;
    global_settings.replace(ctx);
    for n in ctx.db.nodes_world().iter() {
        ctx.db.nodes_world().delete(n);
    }
    let core = Core::from_tnodes(0, &core).to_e_s("Failed to parse Core")?;
    core.save(ctx);
    Ok(())
}

#[reducer]
fn incubator_merge(ctx: &ReducerContext) -> Result<(), String> {
    ctx.is_admin()?;
    if let Ok(houses) = Core::load(ctx).houses_load(ctx) {
        for house in houses {
            house.delete_recursive(ctx);
        }
    }
    for row in ctx.db.incubator_source().iter() {
        ctx.db.incubator_source().delete(row);
    }
    for house in House::collect_children_of_id(ctx, ID_INCUBATOR) {
        house.fill_from_incubator(ctx).clone(ctx, ID_CORE);
    }
    Ok(())
}
