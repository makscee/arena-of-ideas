use nodes_server::NodeKind;

use super::*;

#[table(public, name = nodes)]
pub struct Nodes {
    #[primary_key]
    pub id: u64,
    pub parent: u64,
    pub data: String,
    pub kind: String,
}

#[reducer]
fn r_spawn(ctx: &ReducerContext, kind: String, data: String) -> Result<(), String> {
    let kind = NodeKind::from_str(&kind)
        .map_err(|e| e.to_string())?
        .to_string();
    ctx.db.nodes().insert(Nodes {
        id: next_id(ctx),
        parent: 0,
        data,
        kind,
    });
    Ok(())
}
