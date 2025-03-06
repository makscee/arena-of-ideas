use super::*;

#[reducer]
fn incubator_push(ctx: &ReducerContext, kind: String, datas: Vec<String>) -> Result<(), String> {
    let player = ctx.player()?;
    let kind = NodeKind::from_str(&kind).map_err(|e| e.to_string())?;
    let parent = All::load(ctx).incubator_load(ctx)?.id;
    kind.save_from_strings(ctx, parent, &datas).to_str_err()?;
    Ok(())
}

#[table(public, name = nodes_incubator)]
pub struct TIncubator {
    #[primary_key]
    pub id: u64,
    #[index(btree)]
    pub owner: u64,
}
