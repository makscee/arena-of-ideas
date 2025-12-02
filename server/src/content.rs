use super::*;

#[reducer]
fn content_publish_node(
    ctx: &ReducerContext,
    pack: String,
    parent: Option<u64>,
) -> Result<(), String> {
    let ctx = ctx.as_context();
    info!("Publish node {pack} {parent:?}");
    let player = ctx.player()?;
    let mut pack = ron::from_str::<PackedNodes>(&pack).map_err(|e| e.to_string())?;
    let mut next_id = ctx.next_id();
    pack.reassign_ids(&mut next_id);
    GlobalData::set_next_id(ctx.rctx(), next_id);
    let mut remap: HashMap<u64, u64> = default();
    for (
        id,
        NodeData {
            kind,
            data,
            owner: _,
        },
    ) in &pack.nodes
    {
        let filter = ctx.rctx().db.nodes_world().kind_data_owner();
        if let Some(n) = filter
            .filter((kind, data, ID_CORE))
            .next()
            .or_else(|| filter.filter((kind, data, 0u64)).next())
        {
            remap.insert(*id, n.id);
            continue;
        }
        let kind = kind.to_kind();
        if !kind.is_content() {
            continue;
        }
        let tnode = TNode::new(*id, ID_INCUBATOR, kind, data.clone());
        tnode.insert(ctx.rctx());
        // Record who created this node
        TCreators::record_creation(ctx.rctx(), player.id, *id);
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
        TNodeLink::add_by_id(ctx.rctx(), parent, child, parent_kind, child_kind).track()?;
    }
    if let Some(parent) = parent {
        pack.root.add_parent(ctx.rctx(), parent).track()?;
    }
    Ok(())
}

#[reducer]
fn content_upvote_node(ctx: &ReducerContext, node_id: u64) -> Result<(), String> {
    let player = ctx.as_context().player()?;
    TVotes::upvote_node(ctx, player.id, node_id).map_err(|e| e.to_string())
}

#[reducer]
fn content_downvote_node(ctx: &ReducerContext, node_id: u64) -> Result<(), String> {
    let player = ctx.as_context().player()?;
    TVotes::downvote_node(ctx, player.id, node_id).map_err(|e| e.to_string())
}

#[reducer]
fn content_delete_node(ctx: &ReducerContext, node_id: u64) -> Result<(), String> {
    let ctx = ctx.as_context();
    let player = ctx.player()?;

    if let Some(creator) = ctx.rctx().db.creators().node_id().find(node_id) {
        if creator.player_id != player.id {
            return Err("You can only delete nodes you created".to_string());
        }
    } else {
        return Err("Node creator not found".to_string());
    }

    TNode::delete_by_id_recursive(ctx.rctx(), node_id);

    Ok(())
}

#[reducer]
fn content_suggest_node(ctx: &ReducerContext, kind: String, name: String) -> Result<(), String> {
    let ctx = ctx.as_context();
    let player = ctx.player()?;
    let named_kind = kind
        .parse::<NamedNodeKind>()
        .map_err(|_| format!("Invalid node kind: {}", kind))?;

    let node_id = ctx.next_id();

    let tnode = match named_kind {
        NamedNodeKind::NUnit => NUnit::new(node_id, ID_INCUBATOR, name).to_tnode(),
        NamedNodeKind::NHouse => NHouse::new(node_id, ID_INCUBATOR, name).to_tnode(),
        NamedNodeKind::NAbilityMagic => NAbilityMagic::new(node_id, ID_INCUBATOR, name).to_tnode(),
        NamedNodeKind::NStatusMagic => NStatusMagic::new(node_id, ID_INCUBATOR, name).to_tnode(),
    };

    tnode.insert(ctx.rctx());
    TCreators::record_creation(ctx.rctx(), player.id, node_id);
    GlobalData::set_next_id(ctx.rctx(), node_id + 1);

    Ok(())
}

#[reducer]
fn content_reset_core(ctx: &ReducerContext) -> Result<(), String> {
    info!("Resetting core...");
    ctx.is_admin()?;
    for mut node in ctx.db.nodes_world().owner().filter(ID_CORE) {
        node.owner = ID_INCUBATOR;
        ctx.db.nodes_world().id().update(node);
    }
    Ok(())
}
