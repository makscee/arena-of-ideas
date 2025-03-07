use super::*;

#[reducer]
fn incubator_push(ctx: &ReducerContext, kind: String, datas: Vec<String>) -> Result<(), String> {
    let player = ctx.player()?;
    let kind = NodeKind::from_str(&kind).map_err(|e| e.to_string())?;
    let parent = All::load(ctx).incubator_load(ctx)?.id;
    let nodes = kind.tnode_vec_from_strings(&datas).to_str_err()?;
    for mut node in nodes {
        node.parent = parent;
        node.id = ctx.next_id();
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
