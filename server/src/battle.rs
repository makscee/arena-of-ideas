use super::*;

#[table(public, name = battle)]
struct TBattle {
    #[primary_key]
    id: u64,
    team_left: Vec<TNode>,
    team_right: Vec<TNode>,
}

impl TBattle {
    pub fn init(ctx: &ReducerContext) {}
}

#[reducer]
fn battle_insert(ctx: &ReducerContext, team_left: u64, team_right: u64) -> Result<(), String> {
    TBattle {
        id: next_id(ctx),
        team_left: TNode::gather(ctx, team_left),
        team_right: TNode::gather(ctx, team_right),
    };
    Ok(())
}
