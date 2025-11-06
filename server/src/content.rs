use super::*;

#[reducer]
fn content_publish_node(ctx: &ReducerContext, pack: String) -> Result<(), String> {
    let ctx = ctx.as_context();
    let player = ctx.player()?;
    let mut pack = ron::from_str::<PackedNodes>(&pack).map_err(|e| e.to_string())?;
    let mut next_id = ctx.next_id();
    pack.reassign_ids(&mut next_id);
    GlobalData::set_next_id(ctx.rctx(), next_id);
    let mut remap: HashMap<u64, u64> = default();
    for (id, NodeData { kind, data }) in &pack.nodes {
        if let Some(n) = ctx
            .rctx()
            .db
            .nodes_world()
            .kind_data()
            .filter((kind, data))
            .next()
        {
            remap.insert(*id, n.id);
            continue;
        }
        let kind = kind.to_kind();
        if !kind.is_content() {
            continue;
        }
        let tnode = TNode::new(*id, 0, kind, data.clone());
        tnode.insert(ctx.rctx());
    }
    for NodeLink {
        mut parent,
        mut child,
        parent_kind,
        child_kind,
    } in pack.links
    {
        if !parent_kind.to_kind().is_content() || !child_kind.to_kind().is_content() {
            continue;
        }
        if let Some(id) = remap.get(&parent) {
            parent = *id;
        }
        if let Some(id) = remap.get(&child) {
            child = *id;
        }
        TNodeLink::add_by_id(ctx.rctx(), parent, child, parent_kind, child_kind, false)?;
        TPlayerLinkSelection::select_link(ctx.rctx(), player.id, parent, child)?;
    }
    Ok(())
}

#[reducer]
fn content_vote_node(ctx: &ReducerContext, id: u64, vote: bool) -> Result<(), String> {
    let _ = ctx.as_context().player()?;
    let mut node = id.load_tnode_err(ctx)?;
    let vote = if vote { 1 } else { -1 };
    node.rating += vote;
    node.update(ctx);
    Ok(())
}

#[reducer]
fn content_select_link(ctx: &ReducerContext, parent_id: u64, child_id: u64) -> Result<(), String> {
    TPlayerLinkSelection::select_link(ctx, ctx.as_context().player()?.id, parent_id, child_id)?;
    Ok(())
}

#[reducer]
fn content_deselect_link(
    ctx: &ReducerContext,
    parent_id: u64,
    child_id: u64,
) -> Result<(), String> {
    TPlayerLinkSelection::deselect_link(ctx, ctx.as_context().player()?.id, parent_id, child_id)
        .to_str_err()
}
