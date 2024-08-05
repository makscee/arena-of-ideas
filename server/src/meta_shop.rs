use base_unit::TBaseUnit;
use rand::{seq::IteratorRandom, thread_rng};

use super::*;

#[spacetimedb(table)]
pub struct TMetaShop {
    #[primarykey]
    id: u64,
    item: Item,
    price: i64,
}

impl TMetaShop {
    pub fn init() -> Result<(), String> {
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
fn meta_buy(ctx: ReducerContext, id: u64) -> Result<(), String> {
    let user = ctx.user()?;
    let item = TMetaShop::filter_by_id(&id).context_str("Item not found")?;
    TWallet::change(user.id, -item.price)?;
    item.item.take(user.id)
}
