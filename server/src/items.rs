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

#[spacetimedb(table(public))]
#[derive(Clone)]
pub struct TUnitItem {
    #[primarykey]
    pub id: u64,
    pub owner: u64,
    pub unit: FusedUnit,
}

#[spacetimedb(table(public))]
#[derive(Clone)]
pub struct TUnitShardItem {
    #[primarykey]
    pub id: u64,
    pub owner: u64,
    pub unit: String,
    pub count: u32,
}

#[spacetimedb(table(public))]
#[derive(Clone)]
pub struct TRainbowShardItem {
    #[primarykey]
    pub id: u64,
    pub owner: u64,
    pub count: u32,
}

#[spacetimedb(table(public))]
#[derive(Clone)]
pub struct TLootboxItem {
    #[primarykey]
    pub id: u64,
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
    pub fn from_id(id: u64) -> Result<Self, String> {
        if TUnitItem::filter_by_id(&id).is_some() {
            Ok(Self::Unit)
        } else if TUnitShardItem::filter_by_id(&id).is_some() {
            Ok(Self::UnitShard)
        } else if TRainbowShardItem::filter_by_id(&id).is_some() {
            Ok(Self::RainbowShard)
        } else if TLootboxItem::filter_by_id(&id).is_some() {
            Ok(Self::Lootbox)
        } else {
            Err(format!("Item#{id} not found"))
        }
    }
    pub fn clone_to(self, item_id: u64, owner: u64) -> Result<(), String> {
        match self {
            ItemKind::Unit => {
                let mut item =
                    TUnitItem::filter_by_id(&item_id).context_str("UnitItem not found")?;
                item.id = next_id();
                item.owner = owner;
                TUnitItem::insert(item)?;
            }
            ItemKind::UnitShard => {
                let item = TUnitShardItem::filter_by_id(&item_id)
                    .context_str("UnitShardItem not found")?;
                let mut owner_item = TUnitShardItem::get_or_init(owner, &item.unit);
                owner_item.count += item.count;
                TUnitShardItem::update_by_id(&owner_item.id.clone(), owner_item);
            }
            ItemKind::RainbowShard => {
                let item = TRainbowShardItem::filter_by_id(&item_id)
                    .context_str("RainbowShardItem not found")?;
                let mut owner_item = TRainbowShardItem::get_or_init(owner);
                owner_item.count += item.count;
                TRainbowShardItem::update_by_id(&owner_item.id.clone(), owner_item);
            }
            ItemKind::Lootbox => {
                let item =
                    TLootboxItem::filter_by_id(&item_id).context_str("LootboxItem not found")?;
                let mut owner_item = TLootboxItem::get_or_init(owner, item.kind);
                owner_item.count += item.count;
                TLootboxItem::update_by_id(&owner_item.id.clone(), owner_item);
            }
        }
        Ok(())
    }
    pub fn take(self, item_id: u64, new_owner: u64) -> Result<(), String> {
        match self {
            ItemKind::Unit => {
                let mut item =
                    TUnitItem::filter_by_id(&item_id).context_str("UnitItem not found")?;
                GlobalEvent::ReceiveUnit(item.clone()).post(new_owner);
                item.owner = new_owner;
                TUnitItem::update_by_id(&item_id, item);
            }
            ItemKind::UnitShard => {
                let item = TUnitShardItem::filter_by_id(&item_id)
                    .context_str("UnitShardItem not found")?;
                GlobalEvent::ReceiveUnitShard(item.clone()).post(new_owner);
                let mut owner_item = TUnitShardItem::get_or_init(new_owner, &item.unit);
                owner_item.count += item.count;
                TUnitShardItem::update_by_id(&owner_item.id.clone(), owner_item);
                TUnitShardItem::delete_by_id(&item.id);
            }
            ItemKind::RainbowShard => {
                let item = TRainbowShardItem::filter_by_id(&item_id)
                    .context_str("RainbowShardItem not found")?;
                GlobalEvent::ReceiveRainbowShard(item.clone()).post(new_owner);
                let mut owner_item = TRainbowShardItem::get_or_init(new_owner);
                owner_item.count += item.count;
                TRainbowShardItem::update_by_id(&owner_item.id.clone(), owner_item);
                TRainbowShardItem::delete_by_id(&item.id);
            }
            ItemKind::Lootbox => {
                let item =
                    TLootboxItem::filter_by_id(&item_id).context_str("LootboxItem not found")?;
                GlobalEvent::ReceiveLootbox(item.clone()).post(new_owner);
                let mut owner_item = TLootboxItem::get_or_init(new_owner, item.kind);
                owner_item.count += item.count;
                TLootboxItem::update_by_id(&owner_item.id.clone(), owner_item);
                TLootboxItem::delete_by_id(&item.id);
            }
        }
        Ok(())
    }
    pub fn split(self, item_id: u64, count: u32, new_owner: u64) -> Result<u64, String> {
        match self {
            ItemKind::Unit => {
                if count == 1 {
                    let mut item =
                        TUnitItem::filter_by_id(&item_id).context_str("UnitItem not found")?;
                    item.owner = new_owner;
                    TUnitItem::update_by_id(&item_id, item);
                    Ok(item_id)
                } else {
                    Err("Can't split UnitItem".into())
                }
            }
            ItemKind::UnitShard => {
                let mut item = TUnitShardItem::filter_by_id(&item_id)
                    .context_str("UnitShardItem not found")?;
                if item.count < count {
                    return Err("Insufficient item count".into());
                }
                let mut new_item = item.clone();
                new_item.id = next_id();
                new_item.count = count;
                new_item.owner = new_owner;
                item.count -= count;
                let id = new_item.id;
                TUnitShardItem::insert(new_item).unwrap();
                TUnitShardItem::update_by_id(&item.id.clone(), item);
                Ok(id)
            }
            ItemKind::RainbowShard => {
                let mut item = TRainbowShardItem::filter_by_id(&item_id)
                    .context_str("RainbowShardItem not found")?;
                if item.count < count {
                    return Err("Insufficient item count".into());
                }
                let mut new_item = item.clone();
                new_item.id = next_id();
                new_item.count = count;
                new_item.owner = new_owner;
                item.count -= count;
                let id = new_item.id;
                TRainbowShardItem::insert(new_item).unwrap();
                TRainbowShardItem::update_by_id(&item.id.clone(), item);
                Ok(id)
            }
            ItemKind::Lootbox => {
                let mut item =
                    TLootboxItem::filter_by_id(&item_id).context_str("TLootboxItem not found")?;
                if item.count < count {
                    return Err("Insufficient item count".into());
                }
                let mut new_item = item.clone();
                new_item.id = next_id();
                new_item.count = count;
                new_item.owner = new_owner;
                item.count -= count;
                let id = new_item.id;
                TLootboxItem::insert(new_item).unwrap();
                TLootboxItem::update_by_id(&item.id.clone(), item);
                Ok(id)
            }
        }
    }
}

impl TUnitShardItem {
    pub fn new(owner: u64, unit: String) -> Self {
        Self::insert(Self {
            id: next_id(),
            owner,
            unit,
            count: 1,
        })
        .unwrap()
    }
    fn get_or_init(owner: u64, unit: &str) -> Self {
        Self::filter_by_owner(&owner)
            .find(|i| i.unit.eq(unit))
            .unwrap_or_else(|| {
                Self::insert(Self {
                    id: next_id(),
                    owner,
                    unit: unit.into(),
                    count: 0,
                })
                .unwrap()
            })
    }
}

impl TRainbowShardItem {
    pub fn new(owner: u64) -> Self {
        Self::insert(Self {
            id: next_id(),
            owner,

            count: 1,
        })
        .unwrap()
    }
    fn get_or_init(owner: u64) -> Self {
        Self::filter_by_owner(&owner).next().unwrap_or_else(|| {
            Self::insert(Self {
                id: next_id(),
                owner,
                count: 0,
            })
            .unwrap()
        })
    }
}

impl TLootboxItem {
    pub fn new(owner: u64, kind: LootboxKind) -> Self {
        Self::insert(Self {
            id: next_id(),
            owner,
            kind,
            count: 1,
        })
        .unwrap()
    }
    fn get_or_init(owner: u64, kind: LootboxKind) -> Self {
        Self::filter_by_owner(&owner)
            .find(|i| i.kind == kind)
            .unwrap_or_else(|| {
                Self::insert(Self {
                    id: next_id(),
                    owner,
                    count: 0,
                    kind,
                })
                .unwrap()
            })
    }
}

impl ItemBundle {
    pub fn take(self, owner: u64) -> Result<(), String> {
        for id in self.units {
            let mut unit =
                TUnitItem::filter_by_id(&id).with_context_str(|| format!("Unit {id} not found"))?;
            unit.owner = owner;
            TUnitItem::update_by_id(&unit.id.clone(), unit);
        }
        for id in self.unit_shards {
            let shard = TUnitShardItem::filter_by_id(&id)
                .with_context_str(|| format!("UnitShard {id} not found"))?;
            TUnitShardItem::delete_by_id(&id);
            let mut owner_shard = TUnitShardItem::get_or_init(owner, &shard.unit);
            owner_shard.count += shard.count;
            TUnitShardItem::update_by_id(&owner_shard.id.clone(), owner_shard);
        }
        for id in self.lootboxes {
            let lootbox = TLootboxItem::filter_by_id(&id)
                .with_context_str(|| format!("UnitShard {id} not found"))?;
            TLootboxItem::delete_by_id(&lootbox.id);
            let mut owner_lootbox = TLootboxItem::get_or_init(owner, lootbox.kind);
            owner_lootbox.count += lootbox.count;
            TLootboxItem::update_by_id(&owner_lootbox.id.clone(), owner_lootbox);
        }
        Ok(())
    }
}

#[spacetimedb(reducer)]
fn craft_hero(ctx: ReducerContext, base: String, use_rainbow: u32) -> Result<(), String> {
    let player = ctx.player()?;
    let mut item = TUnitShardItem::get_or_init(player.id, &base);
    let cost = GlobalSettings::get().craft_shards_cost;
    if use_rainbow >= cost {
        return Err("Tried to use too many rainbow shards".into());
    }
    if use_rainbow > 0 {
        let mut item = TRainbowShardItem::get_or_init(player.id);
        if item.count < use_rainbow {
            return Err(format!(
                "Not enough rainbow shards: {} < {use_rainbow}",
                item.count
            ));
        }
        item.count -= use_rainbow;
        TRainbowShardItem::update_by_id(&item.id.clone(), item);
    }
    if item.count + use_rainbow < cost {
        return Err(format!(
            "Not enough shards: {} + {use_rainbow} < {cost}",
            item.count
        ));
    }
    item.count = item.count - cost + use_rainbow;
    TUnitShardItem::update_by_id(&item.id.clone(), item);

    GlobalEvent::CraftUnit(TUnitItem::insert(TUnitItem {
        id: next_id(),
        owner: player.id,
        unit: FusedUnit::from_base_name(base, next_id())?.mutate(),
    })?)
    .post(player.id);
    Ok(())
}

#[spacetimedb(reducer)]
fn dismantle_hero(ctx: ReducerContext, item: u64) -> Result<(), String> {
    let player = ctx.player()?;
    let unit = TUnitItem::filter_by_id(&item).context_str("Item not found")?;
    if unit.owner != player.id {
        return Err(format!("Item not owned by {}", player.id));
    }
    TUnitItem::delete_by_id(&unit.id);
    let mut item = TRainbowShardItem::get_or_init(player.id);
    item.count += unit.unit.rarity() as u32 + 1;
    TRainbowShardItem::update_by_id(&item.id.clone(), item);
    Ok(())
}

#[spacetimedb(reducer)]
fn open_lootbox(ctx: ReducerContext, id: u64) -> Result<(), String> {
    let player = ctx.player()?;
    let mut lootbox = TLootboxItem::filter_by_id(&id).context_str("Lootbox not found")?;
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
    let unit: FusedUnit = TBaseUnit::get_random_for_lootbox(&houses).into();
    let unit = TUnitItem::insert(TUnitItem {
        id: next_id(),
        owner: 0,
        unit: unit.mutate(),
    })?
    .id;
    const AMOUNT: usize = 3;
    let unit_shards = (0..AMOUNT)
        .map(|_| TUnitShardItem {
            id: next_id(),
            owner: 0,
            unit: TBaseUnit::get_random_for_lootbox(&houses).name,
            count: rng().gen_range(1..4),
        })
        .map(|s| {
            let id = s.id;
            TUnitShardItem::insert(s).unwrap();
            id
        })
        .collect_vec();
    let bundle = ItemBundle {
        units: [unit].into(),
        unit_shards,
        lootboxes: default(),
    };
    TTrade::open_lootbox(player.id, bundle)?;
    GlobalEvent::OpenLootbox(lootbox.clone()).post(player.id);
    TLootboxItem::update_by_id(&lootbox.id.clone(), lootbox);
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
