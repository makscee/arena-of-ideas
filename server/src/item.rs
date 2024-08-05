use super::*;
use base_unit::TBaseUnit;
use itertools::Itertools;
use rand::{seq::IteratorRandom, thread_rng, Rng};

#[spacetimedb(table)]
pub struct TItem {
    #[primarykey]
    pub id: u64,
    pub owner: u64,
    pub item: Item,
    pub count: u32,
}

#[derive(SpacetimeType, PartialEq, Clone)]
pub enum Item {
    HeroShard(String),
    Hero(FusedUnit),
    Lootbox,
}

impl Item {
    pub fn take(self, owner: u64) -> Result<(), String> {
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
            Item::Lootbox => {
                let mut item = if let Some(item) = TItem::filter_by_owner(&owner)
                    .filter(|d| d.item.eq(&self))
                    .at_most_one()
                    .map_err(|e| e.to_string())?
                {
                    item
                } else {
                    TItem::insert(TItem {
                        id: next_id(),
                        owner,
                        item: self.clone(),
                        count: 0,
                    })?
                };
                item.count += 1;
                TItem::update_by_id(&item.id.clone(), item);
            }
        };
        Ok(())
    }
}

impl TItem {
    fn change_shards(owner: u64, base: String, delta: i32) -> Result<(), String> {
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
    pub fn craft_hero(owner: u64, base: String) -> Result<(), String> {
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

#[spacetimedb(reducer)]
fn open_lootbox(ctx: ReducerContext, id: u64) -> Result<(), String> {
    let user = ctx.user()?;
    let mut item = TItem::filter_by_id(&id).with_context_str(|| format!("Item not found #{id}"))?;
    if item.owner != user.id {
        return Err(format!("Item #{id} not owned by {}", user.id));
    }
    if item.count == 0 {
        return Err("Lootbox count is 0".into());
    }
    item.count -= 1;
    let amount = thread_rng().gen_range(3..7);
    for _ in 0..amount {
        let name = TBaseUnit::iter().choose(&mut thread_rng()).unwrap().name;
        Item::HeroShard(name).take(user.id)?;
    }
    TItem::update_by_id(&id, item);
    Ok(())
}
