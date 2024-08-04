use super::*;
use itertools::Itertools;

#[spacetimedb(table)]
pub struct TItem {
    #[primarykey]
    pub id: u64,
    pub owner: u64,
    pub item: Item,
    pub count: u32,
}

#[derive(SpacetimeType, PartialEq)]
pub enum Item {
    HeroShard(String),
    Hero(FusedUnit),
}

impl Item {
    pub fn take(self, owner: GID) -> Result<(), String> {
        match &self {
            Item::HeroShard(base) => {
                TItem::change_shards(owner, base.clone(), 1)?;
            }
            Item::Hero(_) => {
                let item = TItem {
                    id: next_id(),
                    owner,
                    item: self,
                    count: 1,
                };
                TItem::insert(item)?;
            }
        };
        Ok(())
    }
}

impl TItem {
    fn change_shards(owner: GID, base: String, delta: i32) -> Result<(), String> {
        let shard_item = Item::HeroShard(base.clone());
        let mut item = if let Some(item) = Self::filter_by_owner(&owner)
            .filter(|d| shard_item.eq(&d.item))
            .at_most_one()
            .map_err(|e| e.to_string())?
        {
            item
        } else {
            Self::insert(Self {
                id: next_id(),
                owner,
                item: shard_item,
                count: 0,
            })?
        };
        if item.count as i32 + delta < 0 {
            return Err("Not enough shards".into());
        }
        item.count = (item.count as i32 + delta) as u32;
        if item.count == 0 {
            Self::delete_by_id(&item.id);
        } else {
            Self::update_by_id(&item.id.clone(), item);
        }
        Ok(())
    }
    pub fn craft_hero(owner: GID, base: String) -> Result<(), String> {
        Self::change_shards(
            owner,
            base.clone(),
            -(GlobalSettings::get().craft_shards_cost as i32),
        )?;
        let id = next_id();
        let hero = FusedUnit::from_base(base, id)?.mutate();
        Self::insert(Self {
            id: next_id(),
            owner,
            item: Item::Hero(hero),
            count: 1,
        })?;
        Ok(())
    }
}

#[spacetimedb(reducer)]
fn craft_hero(ctx: ReducerContext, base: String) -> Result<(), String> {
    let user = ctx.user()?;
    TItem::craft_hero(user.id, base)
}
