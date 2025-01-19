use super::*;

#[table(public, name = battle)]
struct TBattle {
    #[primary_key]
    id: u64,
    team_left: Vec<String>,
    team_right: Vec<String>,
}

#[reducer]
fn battle_insert(
    ctx: &ReducerContext,
    team_left: Vec<String>,
    team_right: Vec<String>,
) -> Result<(), String> {
    let battle = TBattle {
        id: next_id(ctx),
        team_left,
        team_right,
    };
    ctx.db.battle().insert(battle);
    Ok(())
}
