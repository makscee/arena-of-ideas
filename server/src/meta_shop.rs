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
        for _ in 0..3 {
            Self::insert(Self {
                id: next_id(),
                item: Item::HeroShard(
                    TBaseUnit::iter()
                        .choose(&mut thread_rng())
                        .context_str("Failed to choose BaseUnit")?
                        .name,
                ),
                price: 5,
            })?;
        }
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
