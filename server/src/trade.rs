use super::*;

#[spacetimedb::table(public, name = trade)]
pub struct TTrade {
    #[primary_key]
    id: u64,
    a_player: u64,
    b_player: u64,
    a_offer: ItemBundle,
    b_offer: ItemBundle,
    a_accepted: bool,
    b_accepted: bool,
}

#[spacetimedb::reducer]
fn accept_trade(ctx: &ReducerContext, id: u64) -> Result<(), String> {
    let player = ctx.player()?;
    let mut trade: TTrade = ctx
        .db
        .trade()
        .id()
        .find(id)
        .context_str("Trade not found")?;
    if trade.a_player == player.id {
        trade.a_accepted = true;
    }
    if trade.b_player == player.id {
        trade.b_accepted = true;
    } else {
        return Err(format!("player#{} not part of the Trade#{}", player.id, id));
    }
    if trade.a_accepted && trade.b_accepted {
        trade.a_offer.take(ctx, trade.b_player)?;
        trade.b_offer.take(ctx, trade.a_player)?;
        ctx.db.trade().id().delete(id);
    } else {
        ctx.db.trade().id().update(trade);
    }
    Ok(())
}
