use super::*;
use base_unit::TBaseUnit;
use itertools::Itertools;
use rand::{seq::IteratorRandom, Rng};

#[spacetimedb(table(public))]
pub struct TItem {
    #[primarykey]
    pub id: u64,
    pub owner: u64,
    pub stack: ItemStack,
}

#[derive(SpacetimeType, Clone)]
pub struct ItemStack {
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
    pub fn to_stack(self, count: u32) -> ItemStack {
        ItemStack { item: self, count }
    }
}

impl ItemStack {
    pub fn take(self, owner: u64) -> Result<(), String> {
        match &self.item {
            Item::HeroShard(base) => {
                TItem::change_shards(owner, base.clone(), 1)?;
            }
            Item::Hero(_) => {
                let item = TItem {
                    id: next_id(),
                    owner,
                    stack: self,
                };
                TItem::insert(item)?;
            }
            Item::Lootbox => {
                if let Some(mut item) = TItem::filter_by_owner(&owner)
                    .filter(|d| d.stack.item.eq(&self.item))
                    .at_most_one()
                    .map_err(|e| e.to_string())?
                {
                    item.stack.count += self.count;
                    TItem::update_by_id(&item.id.clone(), item);
                } else {
                    TItem::insert(TItem {
                        id: next_id(),
                        owner,
                        stack: self.clone(),
                    })?;
                };
            }
        };
        Ok(())
    }
}

impl TItem {
    fn change_shards(owner: u64, base: String, delta: i32) -> Result<(), String> {
        let shard_item = Item::HeroShard(base.clone());
        let mut item = if let Some(item) = Self::filter_by_owner(&owner)
            .filter(|d| shard_item.eq(&d.stack.item))
            .at_most_one()
            .map_err(|e| e.to_string())?
        {
            item
        } else {
            Self::insert(Self {
                id: next_id(),
                owner,
                stack: ItemStack {
                    item: shard_item,
                    count: 0,
                },
            })?
        };
        if item.stack.count as i32 + delta < 0 {
            return Err("Not enough shards".into());
        }
        item.stack.count = (item.stack.count as i32 + delta) as u32;
        if item.stack.count == 0 {
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
            id,
            owner,
            stack: Item::Hero(hero).to_stack(1),
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
    if item.stack.count == 0 {
        return Err("Lootbox count is 0".into());
    }
    item.stack.count -= 1;
    let amount = rng().gen_range(3..7);
    let items = (0..amount)
        .map(|_| Item::HeroShard(TBaseUnit::iter().choose(&mut rng()).unwrap().name).to_stack(1))
        .collect_vec();
    TTrade::open_lootbox(user.id, id, items)?;
    if item.stack.count == 0 {
        TItem::delete_by_id(&id);
    } else {
        TItem::update_by_id(&id, item);
    }
    Ok(())
}
