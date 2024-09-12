use core::time::Duration;

use super::*;

#[spacetimedb(table(public))]
pub struct TMetaShop {
    #[primarykey]
    id: u64,
    bundle: ItemBundle,
    price: i64,
}

impl TMetaShop {
    pub fn refresh() -> Result<(), String> {
        for Self { id, .. } in Self::iter() {
            Self::delete_by_id(&id);
        }
        let ms = GlobalSettings::get().meta;
        Self::insert(Self {
            id: next_id(),
            bundle: TLootboxItem::new(0, LootboxKind::Regular).into(),
            price: ms.price_lootbox,
        })?;
        for i in TBaseUnit::iter()
            .choose_multiple(&mut rng(), ms.shop_shard_slots as usize)
            .into_iter()
            .map(|u| Self {
                id: next_id(),
                bundle: TUnitShardItem::new(0, u.name).into(),
                price: ms.price_shard,
            })
        {
            Self::insert(i)?;
        }
        Ok(())
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
    item.bundle.take(user.id)
}
