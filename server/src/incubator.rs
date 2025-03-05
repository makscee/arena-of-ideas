use super::*;

#[reducer]
fn incubator_new_node(ctx: &ReducerContext, kind: String, data: String) -> Result<(), String> {
    let player = ctx.player()?;
    let kind = NodeKind::from_str(&kind).map_err(|e| e.to_string())?;
    let mut node = kind.convert(&data).to_str_err()?;
    node.id = ctx.next_id();
    node.parent = All::load(ctx).incubator_load(ctx)?.id;
    ctx.db.nodes_incubator().insert(TIncubator {
        id: node.id,
        owner: player.id,
    });
    ctx.db.nodes_world().insert(node);
    Ok(())
}

#[table(public, name = nodes_incubator)]
pub struct TIncubator {
    #[primary_key]
    pub id: u64,
    #[index(btree)]
    pub owner: u64,
}
