use super::*;

#[spacetimedb(table(public))]
pub struct TTrade {
    #[primarykey]
    id: u64,
    a_user: u64,
    b_user: u64,
    a_offers_items: Vec<ItemStack>,
    b_offers_items: Vec<ItemStack>,
    a_accepted: bool,
    b_accepted: bool,
}

impl TTrade {
    pub fn open_lootbox(owner: u64, id: u64, items: Vec<ItemStack>) -> Result<Self, String> {
        let trade = TTrade {
            id,
            a_user: 0,
            b_user: owner,
            a_offers_items: items,
            b_offers_items: Vec::new(),
            a_accepted: true,
            b_accepted: false,
        };
        TTrade::insert(trade).map_err(|e| e.to_string())
    }
}

#[spacetimedb(reducer)]
fn accept_trade(ctx: ReducerContext, id: u64) -> Result<(), String> {
    let user = ctx.user()?;
    let mut trade = TTrade::filter_by_id(&id).context_str("Trade not found")?;
    if trade.a_user == user.id {
        trade.a_accepted = true;
    }
    if trade.b_user == user.id {
        trade.b_accepted = true;
    } else {
        return Err(format!("User#{} not part of the Trade#{}", user.id, id));
    }
    if trade.a_accepted && trade.b_accepted {
        for item in trade.a_offers_items {
            item.take(trade.b_user)?;
        }
        for item in trade.b_offers_items {
            item.take(trade.a_user)?;
        }
        TTrade::delete_by_id(&id);
    } else {
        TTrade::update_by_id(&id, trade);
    }
    Ok(())
}
