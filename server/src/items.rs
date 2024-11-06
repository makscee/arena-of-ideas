use super::*;

#[derive(SpacetimeType, Clone, Copy)]
pub enum ItemKind {
    Unit,
    UnitShard,
    RainbowShard,
    Lootbox,
}

#[derive(SpacetimeType, Default)]
pub struct ItemBundle {
    pub units: Vec<u64>,
    pub unit_shards: Vec<u64>,
    pub lootboxes: Vec<u64>,
}

#[spacetimedb::table(name = unit_item)]
#[derive(Clone)]
pub struct TUnitItem {
    #[primary_key]
    pub id: u64,
    #[index(btree)]
    pub owner: u64,
    pub unit: FusedUnit,
}

#[spacetimedb::table(name = unit_shard_item)]
#[derive(Clone)]
pub struct TUnitShardItem {
    #[primary_key]
    pub id: u64,
    #[index(btree)]
    pub owner: u64,
    #[index(btree)]
    pub unit: String,
    pub count: u32,
}

#[spacetimedb::table(name = rainbow_shard_item)]
#[derive(Clone)]
pub struct TRainbowShardItem {
    #[primary_key]
    pub id: u64,
    #[index(btree)]
    pub owner: u64,
    pub count: u32,
}

#[spacetimedb::table(name = lootbox_item)]
#[derive(Clone)]
pub struct TLootboxItem {
    #[primary_key]
    pub id: u64,
    #[index(btree)]
    pub owner: u64,
    pub kind: LootboxKind,
    pub count: u32,
}

#[derive(SpacetimeType, Clone, Eq, PartialEq)]
pub enum LootboxKind {
    Regular,
    House(String),
}

impl ItemKind {
    pub fn from_id(ctx: &ReducerContext, id: u64) -> Result<Self, String> {
        if ctx.db.unit_item().id().find(id).is_some() {
            Ok(Self::Unit)
        } else if ctx.db.unit_shard_item().id().find(id).is_some() {
            Ok(Self::UnitShard)
        } else if ctx.db.rainbow_shard_item().id().find(id).is_some() {
            Ok(Self::RainbowShard)
        } else if ctx.db.lootbox_item().id().find(id).is_some() {
            Ok(Self::Lootbox)
        } else {
            Err(format!("Item#{id} not found"))
        }
    }
    pub fn clone_to(self, ctx: &ReducerContext, item_id: u64, owner: u64) -> Result<(), String> {
        match self {
            ItemKind::Unit => {
                let mut item = ctx
                    .db
                    .unit_item()
                    .id()
                    .find(item_id)
                    .context_str("UnitItem not found")?;
                item.id = next_id(ctx);
                item.owner = owner;
                ctx.db.unit_item().insert(item);
            }
            ItemKind::UnitShard => {
                let item = ctx
                    .db
                    .unit_shard_item()
                    .id()
                    .find(item_id)
                    .context_str("UnitShardItem not found")?;
                let mut owner_item = TUnitShardItem::get_or_init(ctx, owner, &item.unit);
                owner_item.count += item.count;
                ctx.db.unit_shard_item().id().update(owner_item);
            }
            ItemKind::RainbowShard => {
                let item = ctx
                    .db
                    .rainbow_shard_item()
                    .id()
                    .find(item_id)
                    .context_str("RainbowShardItem not found")?;
                let mut owner_item = TRainbowShardItem::get_or_init(ctx, owner);
                owner_item.count += item.count;
                ctx.db.rainbow_shard_item().id().update(owner_item);
            }
            ItemKind::Lootbox => {
                let item = ctx
                    .db
                    .lootbox_item()
                    .id()
                    .find(item_id)
                    .context_str("LootboxItem not found")?;
                let mut owner_item = TLootboxItem::get_or_init(ctx, owner, item.kind);
                owner_item.count += item.count;
                ctx.db.lootbox_item().id().update(owner_item);
            }
        }
        Ok(())
    }
    pub fn take(self, ctx: &ReducerContext, item_id: u64, new_owner: u64) -> Result<(), String> {
        match self {
            ItemKind::Unit => {
                let mut item = ctx
                    .db
                    .unit_item()
                    .id()
                    .find(item_id)
                    .context_str("UnitItem not found")?;
                GlobalEvent::ReceiveUnit(item.clone()).post(ctx, new_owner);
                item.owner = new_owner;
                ctx.db.unit_item().id().update(item);
            }
            ItemKind::UnitShard => {
                let item = ctx
                    .db
                    .unit_shard_item()
                    .id()
                    .find(item_id)
                    .context_str("UnitShardItem not found")?;
                GlobalEvent::ReceiveUnitShard(item.clone()).post(ctx, new_owner);
                let mut owner_item = TUnitShardItem::get_or_init(ctx, new_owner, &item.unit);
                owner_item.count += item.count;
                ctx.db.unit_shard_item().id().update(owner_item);
                ctx.db.unit_shard_item().id().delete(item.id);
            }
            ItemKind::RainbowShard => {
                let item = ctx
                    .db
                    .rainbow_shard_item()
                    .id()
                    .find(item_id)
                    .context_str("RainbowShardItem not found")?;
                GlobalEvent::ReceiveRainbowShard(item.clone()).post(ctx, new_owner);
                let mut owner_item = TRainbowShardItem::get_or_init(ctx, new_owner);
                owner_item.count += item.count;
                ctx.db.rainbow_shard_item().id().update(owner_item);
                ctx.db.rainbow_shard_item().id().delete(item.id);
            }
            ItemKind::Lootbox => {
                let item = ctx
                    .db
                    .lootbox_item()
                    .id()
                    .find(item_id)
                    .context_str("LootboxItem not found")?;
                GlobalEvent::ReceiveLootbox(item.clone()).post(ctx, new_owner);
                let mut owner_item = TLootboxItem::get_or_init(ctx, new_owner, item.kind);
                owner_item.count += item.count;
                ctx.db.lootbox_item().id().update(owner_item);
                ctx.db.lootbox_item().id().delete(item.id);
            }
        }
        Ok(())
    }
    pub fn split(
        self,
        ctx: &ReducerContext,
        item_id: u64,
        count: u32,
        new_owner: u64,
    ) -> Result<u64, String> {
        match self {
            ItemKind::Unit => {
                if count == 1 {
                    let mut item = ctx
                        .db
                        .unit_item()
                        .id()
                        .find(item_id)
                        .context_str("UnitItem not found")?;
                    item.owner = new_owner;
                    ctx.db.unit_item().id().update(item);
                    Ok(item_id)
                } else {
                    Err("Can't split UnitItem".into())
                }
            }
            ItemKind::UnitShard => {
                let mut item = ctx
                    .db
                    .unit_shard_item()
                    .id()
                    .find(item_id)
                    .context_str("UnitShardItem not found")?;
                if item.count < count {
                    return Err("Insufficient item count".into());
                }
                let mut new_item = item.clone();
                new_item.id = next_id(ctx);
                new_item.count = count;
                new_item.owner = new_owner;
                item.count -= count;
                let id = new_item.id;
                ctx.db.unit_shard_item().insert(new_item);
                ctx.db.unit_shard_item().id().update(item);
                Ok(id)
            }
            ItemKind::RainbowShard => {
                let mut item = ctx
                    .db
                    .rainbow_shard_item()
                    .id()
                    .find(item_id)
                    .context_str("RainbowShardItem not found")?;
                if item.count < count {
                    return Err("Insufficient item count".into());
                }
                let mut new_item = item.clone();
                new_item.id = next_id(ctx);
                new_item.count = count;
                new_item.owner = new_owner;
                item.count -= count;
                let id = new_item.id;
                ctx.db.rainbow_shard_item().insert(new_item);
                ctx.db.rainbow_shard_item().id().update(item);
                Ok(id)
            }
            ItemKind::Lootbox => {
                let mut item = ctx
                    .db
                    .lootbox_item()
                    .id()
                    .find(item_id)
                    .context_str("TLootboxItem not found")?;
                if item.count < count {
                    return Err("Insufficient item count".into());
                }
                let mut new_item = item.clone();
                new_item.id = next_id(ctx);
                new_item.count = count;
                new_item.owner = new_owner;
                item.count -= count;
                let id = new_item.id;
                ctx.db.lootbox_item().insert(new_item);
                ctx.db.lootbox_item().id().update(item);
                Ok(id)
            }
        }
    }
}

impl TUnitShardItem {
    pub fn new(ctx: &ReducerContext, owner: u64, unit: String) -> Self {
        ctx.db.unit_shard_item().insert(Self {
            id: next_id(ctx),
            owner,
            unit,
            count: 1,
        })
    }
    fn get_or_init(ctx: &ReducerContext, owner: u64, unit: &str) -> Self {
        ctx.db
            .unit_shard_item()
            .owner()
            .filter(owner)
            .find(|i| i.unit.eq(unit))
            .unwrap_or_else(|| {
                ctx.db.unit_shard_item().insert(Self {
                    id: next_id(ctx),
                    owner,
                    unit: unit.into(),
                    count: 0,
                })
            })
    }
}

impl TRainbowShardItem {
    pub fn new(ctx: &ReducerContext, owner: u64) -> Self {
        ctx.db.rainbow_shard_item().insert(Self {
            id: next_id(ctx),
            owner,
            count: 1,
        })
    }
    fn get_or_init(ctx: &ReducerContext, owner: u64) -> Self {
        ctx.db
            .rainbow_shard_item()
            .owner()
            .filter(owner)
            .next()
            .unwrap_or_else(|| {
                ctx.db.rainbow_shard_item().insert(Self {
                    id: next_id(ctx),
                    owner,
                    count: 0,
                })
            })
    }
}

impl TLootboxItem {
    pub fn new(ctx: &ReducerContext, owner: u64, kind: LootboxKind) -> Self {
        ctx.db.lootbox_item().insert(Self {
            id: next_id(ctx),
            owner,
            kind,
            count: 1,
        })
    }
    fn get_or_init(ctx: &ReducerContext, owner: u64, kind: LootboxKind) -> Self {
        ctx.db
            .lootbox_item()
            .owner()
            .filter(owner)
            .find(|i| i.kind == kind)
            .unwrap_or_else(|| {
                ctx.db.lootbox_item().insert(Self {
                    id: next_id(ctx),
                    owner,
                    count: 0,
                    kind,
                })
            })
    }
}

impl ItemBundle {
    pub fn take(self, ctx: &ReducerContext, owner: u64) -> Result<(), String> {
        for id in self.units {
            let mut unit = ctx
                .db
                .unit_item()
                .id()
                .find(id)
                .with_context_str(|| format!("Unit {id} not found"))?;
            unit.owner = owner;
            ctx.db.unit_item().id().update(unit);
        }
        for id in self.unit_shards {
            let shard = ctx
                .db
                .unit_shard_item()
                .id()
                .find(id)
                .with_context_str(|| format!("UnitShard {id} not found"))?;
            ctx.db.unit_shard_item().id().delete(id);

            let mut owner_shard = TUnitShardItem::get_or_init(ctx, owner, &shard.unit);
            owner_shard.count += shard.count;
            ctx.db.unit_shard_item().id().update(owner_shard);
        }
        for id in self.lootboxes {
            let lootbox = ctx
                .db
                .lootbox_item()
                .id()
                .find(id)
                .with_context_str(|| format!("UnitShard {id} not found"))?;
            ctx.db.lootbox_item().id().delete(lootbox.id);
            let mut owner_lootbox = TLootboxItem::get_or_init(ctx, owner, lootbox.kind);
            owner_lootbox.count += lootbox.count;
            ctx.db.lootbox_item().id().update(owner_lootbox);
        }
        Ok(())
    }
}

#[spacetimedb::reducer]
fn craft_hero(ctx: &ReducerContext, base: String, use_rainbow: u32) -> Result<(), String> {
    let player = ctx.player()?;
    let mut item = TUnitShardItem::get_or_init(ctx, player.id, &base);
    let cost = GlobalSettings::get(ctx).craft_shards_cost;
    if use_rainbow >= cost {
        return Err("Tried to use too many rainbow shards".into());
    }
    if use_rainbow > 0 {
        let mut item = TRainbowShardItem::get_or_init(ctx, player.id);
        if item.count < use_rainbow {
            return Err(format!(
                "Not enough rainbow shards: {} < {use_rainbow}",
                item.count
            ));
        }
        item.count -= use_rainbow;
        ctx.db.rainbow_shard_item().id().update(item);
    }
    if item.count + use_rainbow < cost {
        return Err(format!(
            "Not enough shards: {} + {use_rainbow} < {cost}",
            item.count
        ));
    }
    item.count = item.count - cost + use_rainbow;
    ctx.db.unit_shard_item().id().update(item);

    GlobalEvent::CraftUnit(ctx.db.unit_item().insert(TUnitItem {
        id: next_id(ctx),
        owner: player.id,
        unit: FusedUnit::from_base_name(ctx, base, next_id(ctx))?.mutate(ctx),
    }))
    .post(ctx, player.id);
    Ok(())
}

#[spacetimedb::reducer]
fn dismantle_hero(ctx: &ReducerContext, item: u64) -> Result<(), String> {
    let player = ctx.player()?;
    let unit = ctx
        .db
        .unit_item()
        .id()
        .find(item)
        .context_str("Item not found")?;
    if unit.owner != player.id {
        return Err(format!("Item not owned by {}", player.id));
    }
    ctx.db.unit_item().id().delete(unit.id);
    let mut item = TRainbowShardItem::get_or_init(ctx, player.id);
    item.count += unit.unit.rarity(ctx) as u32 + 1;
    ctx.db.rainbow_shard_item().id().update(item);
    Ok(())
}

#[spacetimedb::reducer]
fn open_lootbox(ctx: &ReducerContext, id: u64) -> Result<(), String> {
    let player = ctx.player()?;
    let mut lootbox = ctx
        .db
        .lootbox_item()
        .id()
        .find(id)
        .context_str("Lootbox not found")?;
    if lootbox.owner != player.id {
        return Err("Tried to open lootbox that is not owned".into());
    }
    if lootbox.count == 0 {
        return Err("No lootbox owned".into());
    }
    lootbox.count -= 1;
    let houses = match &lootbox.kind {
        LootboxKind::House(house) => [house.clone()].into(),
        LootboxKind::Regular => default(),
    };
    let unit: FusedUnit = TBaseUnit::get_random_for_lootbox(ctx, &houses).into_fused(ctx);
    let unit = ctx
        .db
        .unit_item()
        .insert(TUnitItem {
            id: next_id(ctx),
            owner: 0,
            unit: unit.mutate(ctx),
        })
        .id;
    const AMOUNT: usize = 3;
    let unit_shards = (0..AMOUNT)
        .map(|_| TUnitShardItem {
            id: next_id(ctx),
            owner: 0,
            unit: TBaseUnit::get_random_for_lootbox(ctx, &houses).name,
            count: ctx.rng().gen_range(1..4),
        })
        .map(|s| {
            let id = s.id;
            ctx.db.unit_shard_item().insert(s);
            id
        })
        .collect_vec();
    let bundle = ItemBundle {
        units: [unit].into(),
        unit_shards,
        lootboxes: default(),
    };
    TTrade::open_lootbox(ctx, player.id, bundle);
    GlobalEvent::OpenLootbox(lootbox.clone()).post(ctx, player.id);
    ctx.db.lootbox_item().id().update(lootbox);
    Ok(())
}

impl From<TUnitItem> for ItemBundle {
    fn from(value: TUnitItem) -> Self {
        Self {
            units: [value.id].into(),
            unit_shards: default(),
            lootboxes: default(),
        }
    }
}
impl From<TUnitShardItem> for ItemBundle {
    fn from(value: TUnitShardItem) -> Self {
        Self {
            units: default(),
            unit_shards: [value.id].into(),
            lootboxes: default(),
        }
    }
}
impl From<TLootboxItem> for ItemBundle {
    fn from(value: TLootboxItem) -> Self {
        Self {
            units: default(),
            unit_shards: default(),
            lootboxes: [value.id].into(),
        }
    }
}
