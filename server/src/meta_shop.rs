use base_unit::base_unit;
use house::house;
use spacetimedb::Table;

use super::*;

#[spacetimedb::table(name = meta_shop)]
#[derive(Clone, Copy)]
pub struct TMetaShop {
    #[primary_key]
    id: u64,
    item_kind: ItemKind,
    price: i64,
}

impl TMetaShop {
    pub fn refresh(ctx: &ReducerContext) -> Result<(), String> {
        for Self { id, .. } in ctx.db.meta_shop().iter() {
            ctx.db.meta_shop().id().delete(id);
        }
        let ms = GlobalSettings::get(ctx).meta;
        ctx.db.meta_shop().insert(Self {
            id: TLootboxItem::new(ctx, 0, LootboxKind::Regular).id,
            item_kind: ItemKind::Lootbox,
            price: ms.price_lootbox,
        });
        let house = ctx.db.house().iter().choose(&mut ctx.rng()).unwrap().name;
        ctx.db.meta_shop().insert(Self {
            id: TLootboxItem::new(ctx, 0, LootboxKind::House(house)).id,
            item_kind: ItemKind::Lootbox,
            price: ms.price_lootbox,
        });
        for i in ctx
            .db
            .base_unit()
            .iter()
            .filter(|u| u.pool == UnitPool::Game)
            .choose_multiple(&mut ctx.rng(), ms.shop_shard_slots as usize)
            .into_iter()
            .map(|u| Self {
                id: TUnitShardItem::new(ctx, 0, u.name).id,
                item_kind: ItemKind::UnitShard,
                price: ms.price_shard,
            })
        {
            ctx.db.meta_shop().insert(i);
        }
        Ok(())
    }
    fn take(self, ctx: &ReducerContext, owner: u64) -> Result<(), String> {
        self.item_kind.clone_to(ctx, self.id, owner)
    }
}

#[spacetimedb::reducer]
fn meta_buy(ctx: &ReducerContext, id: u64) -> Result<(), String> {
    let player = ctx.player()?;
    let mut item = ctx
        .db
        .meta_shop()
        .id()
        .find(id)
        .context_str("Item not found")?;
    let mut price = item.price;

    if TDailyState::get(ctx, player.id).meta_shop_discount(ctx) {
        price = (price as f32 * GlobalSettings::get(ctx).meta.daily_discount) as i64;
    }
    TWallet::change(ctx, player.id, -price)?;
    match item.item_kind {
        ItemKind::UnitShard | ItemKind::Unit | ItemKind::RainbowShard => {
            item.price += 1;
            ctx.db.meta_shop().id().update(item);
        }
        ItemKind::Lootbox => {}
    };
    GlobalEvent::MetaShopBuy(item.clone()).post(ctx, player.id);
    item.take(ctx, player.id)
}
