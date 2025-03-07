use super::*;

#[reducer]
fn incubator_push(ctx: &ReducerContext, kind: String, datas: Vec<String>) -> Result<(), String> {
    let player = ctx.player()?;
    let kind = NodeKind::from_str(&kind).map_err(|e| e.to_string())?;
    let parent = All::load(ctx).incubator_load(ctx)?.id;
    let nodes = kind.tnode_vec_from_strings(ctx, &datas).to_str_err()?;
    let nodes: HashMap<u64, TNode> = HashMap::from_iter(nodes.into_iter().map(|n| (n.id, n)));
    let link_kinds = NodeKind::get_incubator_links();
    for (_, node) in nodes.iter() {
        let id = node.id;
        let parent_id = node.parent;
        let kind = NodeKind::from_str(&node.kind).unwrap();
        let Some(parent_node) = nodes.get(&parent_id) else {
            continue;
        };
        let parent_kind = NodeKind::from_str(&parent_node.kind).unwrap();
        if link_kinds
            .get(&kind)
            .is_some_and(|links| links.contains(&parent_kind))
        {
            ctx.db.incubator_links().insert(TIncubatorLinks {
                from: id,
                to: parent_id,
                to_kind: parent_kind.to_string(),
                score: 1,
            });
        } else if link_kinds
            .get(&parent_kind)
            .is_some_and(|links| links.contains(&kind))
        {
            ctx.db.incubator_links().insert(TIncubatorLinks {
                from: parent_id,
                to: id,
                to_kind: kind.to_string(),
                score: 1,
            });
        }
    }
    for mut node in nodes.into_values() {
        node.parent = parent;
        ctx.db.nodes_world().insert(node);
    }
    Ok(())
}

#[table(public, name = incubator_nodes)]
pub struct TIncubator {
    #[primary_key]
    pub id: u64,
    #[index(btree)]
    pub owner: u64,
}

#[table(public, name = incubator_links)]
pub struct TIncubatorLinks {
    pub from: u64,
    pub to: u64,
    pub to_kind: String,
    pub score: i64,
}
