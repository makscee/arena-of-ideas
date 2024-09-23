use super::*;

#[spacetimedb(table(public))]
pub struct TAuction {
    #[primarykey]
    pub item_id: u64,
    pub owner: u64,
    pub item_kind: ItemKind,
    pub price: i64,
}

#[spacetimedb(reducer)]
fn auction_create(ctx: ReducerContext, item_id: u64, count: u32, price: i64) -> Result<(), String> {
    let user = ctx.user()?;
    let item_kind = ItemKind::from_id(item_id)?;
    let new_item = item_kind.split(item_id, count, 0)?;
    TAuction::insert(TAuction {
        item_id: new_item,
        owner: user.id,
        item_kind,
        price,
    })
    .map_err(|e| e.to_string())?;
    Ok(())
}
#[spacetimedb(reducer)]
fn auction_buy(ctx: ReducerContext, item_id: u64) -> Result<(), String> {
    let user = ctx.user()?;
    let auction = TAuction::filter_by_item_id(&item_id)
        .with_context_str(|| format!("Action#{item_id} not found"))?;
    TWallet::change(auction.owner, auction.price)?;
    TWallet::change(user.id, -auction.price)?;
    let item_kind = ItemKind::from_id(item_id)?;
    item_kind.take(item_id, user.id)?;
    TAuction::delete_by_item_id(&item_id);
    Ok(())
}
#[spacetimedb(reducer)]
fn auction_cancel(ctx: ReducerContext, item_id: u64) -> Result<(), String> {
    let user = ctx.user()?;
    let auction = TAuction::filter_by_item_id(&item_id)
        .with_context_str(|| format!("Action#{item_id} not found"))?;
    if auction.owner != user.id {
        return Err(format!("Action#{item_id} not owned by {}", user.id));
    }
    TAuction::delete_by_item_id(&item_id);
    let item_kind = ItemKind::from_id(item_id)?;
    item_kind.take(item_id, user.id)
}
