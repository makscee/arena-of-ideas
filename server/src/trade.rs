use super::*;

#[spacetimedb(table(public))]
pub struct TTrade {
    #[primarykey]
    id: u64,
    a_player: u64,
    b_player: u64,
    a_offer: ItemBundle,
    b_offer: ItemBundle,
    a_accepted: bool,
    b_accepted: bool,
}

impl TTrade {
    pub fn open_lootbox(owner: u64, bundle: ItemBundle) -> Result<Self, String> {
        let trade = TTrade {
            id: next_id(),
            a_player: 0,
            b_player: owner,
            a_offer: bundle,
            b_offer: default(),
            a_accepted: true,
            b_accepted: false,
        };
        TTrade::insert(trade).map_err(|e| e.to_string())
    }
}

#[spacetimedb(reducer)]
fn accept_trade(ctx: ReducerContext, id: u64) -> Result<(), String> {
    let player = ctx.player()?;
    let mut trade = TTrade::filter_by_id(&id).context_str("Trade not found")?;
    if trade.a_player == player.id {
        trade.a_accepted = true;
    }
    if trade.b_player == player.id {
        trade.b_accepted = true;
    } else {
        return Err(format!("player#{} not part of the Trade#{}", player.id, id));
    }
    if trade.a_accepted && trade.b_accepted {
        trade.a_offer.take(trade.b_player)?;
        trade.b_offer.take(trade.a_player)?;
        TTrade::delete_by_id(&id);
    } else {
        TTrade::update_by_id(&id, trade);
    }
    Ok(())
}
