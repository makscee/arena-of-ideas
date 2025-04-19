use super::*;

#[reducer]
fn sync_assets(
    ctx: &ReducerContext,
    global_settings: GlobalSettings,
    core: Vec<TNode>,
    players: Vec<TNode>,
    incubator: Vec<TNode>,
    incubator_nodes: Vec<TIncubator>,
    incubator_links: Vec<TIncubatorLinks>,
    incubator_votes: Vec<TIncubatorVotes>,
) -> Result<(), String> {
    GlobalData::init(ctx);
    ctx.is_admin()?;
    global_settings.replace(ctx);

    let core = NCore::from_tnodes(ID_CORE, &core).to_e_s("Failed to parse NCore")?;
    let players = NPlayers::from_tnodes(ID_PLAYERS, &players).to_e_s("Failed to parse Players")?;
    let incubator =
        NIncubator::from_tnodes(ID_INCUBATOR, &incubator).to_e_s("Failed to parse Incubator")?;
    for n in ctx.db.nodes_world().iter() {
        ctx.db.nodes_world().delete(n);
    }

    for n in core.to_tnodes() {
        ctx.db.nodes_world().insert(n);
    }
    for n in players.to_tnodes() {
        ctx.db.nodes_world().insert(n);
    }
    for n in incubator.to_tnodes() {
        ctx.db.nodes_world().insert(n);
    }

    for n in ctx.db.incubator_nodes().iter() {
        ctx.db.incubator_nodes().delete(n);
    }
    for n in incubator_nodes {
        ctx.db.incubator_nodes().insert(n);
    }
    for n in ctx.db.incubator_links().iter() {
        ctx.db.incubator_links().delete(n);
    }
    for n in incubator_links {
        ctx.db.incubator_links().insert(n);
    }
    for n in ctx.db.incubator_votes().iter() {
        ctx.db.incubator_votes().delete(n);
    }
    for n in incubator_votes {
        ctx.db.incubator_votes().insert(n);
    }
    Ok(())
}

#[reducer]
fn incubator_merge(ctx: &ReducerContext) -> Result<(), String> {
    ctx.is_admin()?;
    if let Ok(houses) = NCore::load(ctx).houses_load(ctx) {
        for house in houses {
            house.delete_recursive(ctx);
        }
    }
    for row in ctx.db.incubator_source().iter() {
        ctx.db.incubator_source().delete(row);
    }
    let mut remap: HashMap<u64, u64> = default();
    for house in NHouse::collect_children_of_id(ctx, ID_INCUBATOR) {
        house
            .fill_from_incubator(ctx)
            .clone(ctx, ID_CORE, &mut remap);
    }
    for (from, to) in remap {
        ctx.db.incubator_source().insert(TIncubatorSource {
            node_id: to,
            incubator_id: from,
        });
    }
    Ok(())
}
