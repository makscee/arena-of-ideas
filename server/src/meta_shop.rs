use super::*;

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
        for i in TBaseUnit::iter()
            .choose_multiple(&mut thread_rng(), 3)
            .into_iter()
            .map(|u| Self {
                id: next_id(),
                item: Item::HeroShard(u.name),
                price: 5,
            })
        {
            Self::insert(i)?;
        }
        Self::insert(Self {
            id: next_id(),
            item: Item::Lootbox,
            price: 15,
        })?;
        Ok(())
    }
}

#[spacetimedb(reducer)]
pub fn meta_shop_refresh() -> Result<(), String> {
    schedule!("30s", meta_shop_refresh());
    let last_refresh = GlobalData::get().last_shop_refresh;
    let since = Timestamp::now()
        .duration_since(last_refresh)
        .map_err(|_| "Elapsed duration get error".to_owned())?;
    if since.as_secs() < 25 {
        return Err("Not enough time passed".to_owned());
    }
    TMetaShop::refresh()?;
    GlobalData::register_shop_refresh();
    Ok(())
}

#[spacetimedb(reducer)]
fn meta_buy(ctx: ReducerContext, id: u64) -> Result<(), String> {
    let user = ctx.user()?;
    let item = TMetaShop::filter_by_id(&id).context_str("Item not found")?;
    TWallet::change(user.id, -item.price)?;
    item.item.take(user.id)
}
