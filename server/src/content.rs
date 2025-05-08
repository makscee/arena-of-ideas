use super::*;

#[reducer]
fn core_publish(ctx: &ReducerContext, pack: String) -> Result<(), String> {
    let player = ctx.player()?;
    let mut pack = ron::from_str::<PackedNodes>(&pack).map_err(|e| e.to_string())?;
    let mut next_id = ctx.next_id();
    pack.reassign_ids(&mut next_id);
    GlobalData::set_next_id(ctx, next_id);
    for (id, NodeData { kind, data }) in &pack.nodes {
        let tnode = TNode::new(*id, 0, kind.to_kind(), data.clone());
        tnode.insert(ctx);
    }
    for NodeLink {
        parent,
        child,
        parent_kind,
        child_kind,
    } in pack.links
    {
        TNodeLink::add_by_id(ctx, parent, child, parent_kind, child_kind)?;
    }
    Ok(())
}
