use core::time::Duration;

use super::*;

#[spacetimedb(table(public))]
pub struct TMetaShop {
    #[primarykey]
    id: u64,
    item_kind: ItemKind,
    price: i64,
}

impl TMetaShop {
    pub fn refresh() -> Result<(), String> {
        for Self { id, .. } in Self::iter() {
            Self::delete_by_id(&id);
        }
        let ms = GlobalSettings::get().meta;
        Self::insert(Self {
            id: TLootboxItem::new(0, LootboxKind::Regular).id,
            item_kind: ItemKind::Lootbox,
            price: ms.price_lootbox,
        })?;
        for i in TBaseUnit::iter()
            .choose_multiple(&mut rng(), ms.shop_shard_slots as usize)
            .into_iter()
            .map(|u| Self {
                id: TUnitShardItem::new(0, u.name).id,
                item_kind: ItemKind::UnitShard,
                price: ms.price_shard,
            })
        {
            Self::insert(i)?;
        }
        Ok(())
    }
    fn take(self, owner: u64) -> Result<(), String> {
        self.item_kind.clone_to(self.id, owner)
    }
}

pub fn meta_shop_refresh() -> Result<(), String> {
    spacetimedb::println!("Refresh start");
    let last_refresh = GlobalData::get().last_shop_refresh;
    let since = Timestamp::now()
        .duration_since(last_refresh)
        .map(|d| d.as_secs())
        .unwrap_or(u64::MAX);
    let period = GlobalSettings::get().meta.shop_refresh_period_secs;
    if since < period {
        let time = last_refresh + Duration::from_secs(period);
        spacetimedb::println!("Refresh reschedule {time:?}");
        return Ok(());
    }
    TMetaShop::refresh()?;
    GlobalData::register_shop_refresh();
    spacetimedb::println!("Refresh success");
    Ok(())
}

#[spacetimedb(reducer)]
fn meta_buy(ctx: ReducerContext, id: u64) -> Result<(), String> {
    let user = ctx.user()?;
    let item = TMetaShop::filter_by_id(&id).context_str("Item not found")?;
    TWallet::change(user.id, -item.price)?;
    item.take(user.id)
}
