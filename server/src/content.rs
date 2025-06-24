use super::*;

use raw_nodes::NodeKindExt;

#[reducer]
fn content_publish_node(ctx: &ReducerContext, pack: String) -> Result<(), String> {
    let _ = ctx.player()?;
    let mut pack = ron::from_str::<PackedNodes>(&pack).map_err(|e| e.to_string())?;
    let mut next_id = ctx.next_id();
    pack.reassign_ids(&mut next_id);
    GlobalData::set_next_id(ctx, next_id);
    let mut remap: HashMap<u64, u64> = default();
    for (id, NodeData { kind, data }) in &pack.nodes {
        if let Some(n) = ctx.db.nodes_world().kind_data().filter((kind, data)).next() {
            remap.insert(*id, n.id);
            continue;
        }
        let tnode = TNode::new(*id, 0, kind.to_kind(), data.clone());
        tnode.insert(ctx);
    }
    for NodeLink {
        mut parent,
        mut child,
        parent_kind,
        child_kind,
    } in pack.links
    {
        if let Some(id) = remap.get(&parent) {
            parent = *id;
        }
        if let Some(id) = remap.get(&child) {
            child = *id;
        }
        let _ = TNodeLink::add_by_id(ctx, parent, child, parent_kind, child_kind, false);
    }
    Ok(())
}

#[reducer]
fn content_vote_node(ctx: &ReducerContext, id: u64, vote: bool) -> Result<(), String> {
    let _ = ctx.player()?;
    let mut node = id.find_err(ctx)?;
    let vote = if vote { 1 } else { -1 };
    node.rating += vote;
    node.update(ctx);
    Ok(())
}

#[reducer]
fn content_vote_link(
    ctx: &ReducerContext,
    parent: u64,
    child: u64,
    vote: bool,
) -> Result<(), String> {
    let _ = ctx.player()?;
    let parent = parent.find_err(ctx)?;
    let child = child.find_err(ctx)?;
    TNodeLink::vote(ctx, &parent, &child, vote);
    Ok(())
}
