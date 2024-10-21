use house::THouse;

use super::*;

#[spacetimedb(table(public))]
#[derive(Clone, Copy)]
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
        let house = THouse::iter().choose(&mut rng()).unwrap().name;
        Self::insert(Self {
            id: TLootboxItem::new(0, LootboxKind::House(house)).id,
            item_kind: ItemKind::Lootbox,
            price: ms.price_lootbox,
        })?;
        for i in TBaseUnit::iter()
            .filter(|u| u.rarity >= 0)
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

#[spacetimedb(reducer)]
fn meta_buy(ctx: ReducerContext, id: u64) -> Result<(), String> {
    let user = ctx.user()?;
    let mut item = TMetaShop::filter_by_id(&id).context_str("Item not found")?;
    TWallet::change(user.id, -item.price)?;
    match item.item_kind {
        ItemKind::UnitShard | ItemKind::Unit | ItemKind::RainbowShard => {
            item.price += 1;
            TMetaShop::update_by_id(&id, item);
        }
        ItemKind::Lootbox => {}
    };
    item.take(user.id)
}
