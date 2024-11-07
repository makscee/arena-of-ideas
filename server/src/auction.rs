use spacetimedb::Table;

use super::*;

#[spacetimedb::table(public, name = auction)]
#[derive(Clone)]
pub struct TAuction {
    #[primary_key]
    pub item_id: u64,
    pub owner: u64,
    pub item_kind: ItemKind,
    pub price: i64,
}

#[spacetimedb::reducer]
fn auction_create(
    ctx: &ReducerContext,
    item_id: u64,
    count: u32,
    price: i64,
) -> Result<(), String> {
    let player = ctx.player()?;
    let item_kind = ItemKind::from_id(ctx, item_id)?;
    let new_item = item_kind.split(ctx, item_id, count, 0)?;
    let auction = TAuction {
        item_id: new_item,
        owner: player.id,
        item_kind,
        price,
    };
    GlobalEvent::AuctionPost(auction.clone()).post(ctx, player.id);
    ctx.db.auction().insert(auction);
    Ok(())
}
#[spacetimedb::reducer]
fn auction_buy(ctx: &ReducerContext, item_id: u64) -> Result<(), String> {
    let player = ctx.player()?;
    let auction = ctx
        .db
        .auction()
        .item_id()
        .find(item_id)
        .with_context_str(|| format!("Action#{item_id} not found"))?;
    TWallet::change(ctx, auction.owner, auction.price)?;
    TWallet::change(ctx, player.id, -auction.price)?;
    let item_kind = ItemKind::from_id(ctx, item_id)?;
    item_kind.take(ctx, item_id, player.id)?;
    ctx.db.auction().item_id().delete(item_id);
    GlobalEvent::AuctionBuy(auction).post(ctx, player.id);
    Ok(())
}
#[spacetimedb::reducer]
fn auction_cancel(ctx: &ReducerContext, item_id: u64) -> Result<(), String> {
    let player = ctx.player()?;
    let auction = ctx
        .db
        .auction()
        .item_id()
        .find(item_id)
        .with_context_str(|| format!("Action#{item_id} not found"))?;
    if auction.owner != player.id {
        return Err(format!("Action#{item_id} not owned by {}", player.id));
    }
    ctx.db.auction().item_id().delete(item_id);
    GlobalEvent::AuctionCancel(auction).post(ctx, player.id);
    let item_kind = ItemKind::from_id(ctx, item_id)?;
    item_kind.take(ctx, item_id, player.id)
}
