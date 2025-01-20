use super::*;

#[reducer]
fn match_buy(ctx: &ReducerContext, slot: u8) -> Result<(), String> {
    let m = Match::from_table(ctx, 0);
    Ok(())
}
