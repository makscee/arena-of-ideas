use core::time::Duration;

use spacetimedb::sys::schedule;

use super::*;

#[derive(SpacetimeType)]
pub struct MetaSettings {
    price_lootbox: i64,
    price_shard: i64,
    shop_refresh_period_secs: u64,
    shop_shard_slots: u32,
}

#[spacetimedb(table)]
pub struct TMetaShop {
    #[primarykey]
    id: u64,
    item: Item,
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
            item: Item::Lootbox,
            price: ms.price_lootbox,
        })?;
        for i in TBaseUnit::iter()
            .choose_multiple(&mut thread_rng(), ms.shop_shard_slots as usize)
            .into_iter()
            .map(|u| Self {
                id: next_id(),
                item: Item::HeroShard(u.name),
                price: ms.price_shard,
            })
        {
            Self::insert(i)?;
        }
        Ok(())
    }
}

#[spacetimedb(reducer)]
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
        schedule("meta_shop_refresh", &[], time.into_micros_since_epoch());
        return Ok(());
    }
    schedule(
        "meta_shop_refresh",
        &[],
        (Timestamp::now() + Duration::from_secs(period)).into_micros_since_epoch(),
    );
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
    item.item.take(user.id)
}
