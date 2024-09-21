use super::*;

#[derive(SpacetimeType)]
pub enum ItemKind {
    Unit,
    UnitShard,
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
pub struct TLootboxItem {
    #[primarykey]
    pub id: u64,
    pub owner: u64,
    pub kind: LootboxKind,
    pub count: u32,
}

#[derive(SpacetimeType, Copy, Clone, Eq, PartialEq)]
pub enum LootboxKind {
    Regular,
}

impl ItemKind {
    pub fn clone_to(self, id: u64, owner: u64) -> Result<(), String> {
        match self {
            ItemKind::Unit => {
                let mut item = TUnitItem::filter_by_id(&id).context_str("UnitItem not found")?;
                item.id = next_id();
                item.owner = owner;
                TUnitItem::insert(item)?;
            }
            ItemKind::UnitShard => {
                let item =
                    TUnitShardItem::filter_by_id(&id).context_str("UnitShardItem not found")?;
                let mut owner_item = TUnitShardItem::get_or_init(owner, &item.unit);
                owner_item.count += item.count;
                TUnitShardItem::update_by_id(&owner_item.id.clone(), owner_item);
            }
            ItemKind::Lootbox => {
                let item =
                    TLootboxItem::filter_by_id(&id).context_str("UnitShardItem not found")?;
                let mut owner_item = TLootboxItem::get_or_init(owner, item.kind);
                owner_item.count += item.count;
                TLootboxItem::update_by_id(&owner_item.id.clone(), owner_item);
            }
        }
        Ok(())
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
fn craft_hero(ctx: ReducerContext, base: String) -> Result<(), String> {
    let user = ctx.user()?;
    let mut item = TUnitShardItem::get_or_init(user.id, &base);
    let cost = GlobalSettings::get().craft_shards_cost;
    if item.count < cost {
        return Err(format!("Not enough shards: {} < {cost}", item.count));
    }
    item.count -= cost;
    TUnitShardItem::update_by_id(&item.id.clone(), item);
    TUnitItem::insert(TUnitItem {
        id: next_id(),
        owner: user.id,
        unit: FusedUnit::from_base_name(base, next_id())?.mutate(),
    })?;
    Ok(())
}

#[spacetimedb(reducer)]
fn open_lootbox(ctx: ReducerContext, id: u64) -> Result<(), String> {
    let user = ctx.user()?;
    let mut lootbox = TLootboxItem::filter_by_id(&id).context_str("Lootbox not found")?;
    if lootbox.owner != user.id {
        return Err("Tried to open lootbox that is not owned".into());
    }
    if lootbox.count == 0 {
        return Err("No lootbox owned".into());
    }
    lootbox.count -= 1;
    match lootbox.kind {
        LootboxKind::Regular => {
            let unit: FusedUnit = TBaseUnit::get_random_for_lootbox().into();
            let unit = TUnitItem::insert(TUnitItem {
                id: next_id(),
                owner: user.id,
                unit: unit.mutate(),
            })?
            .id;
            const AMOUNT: usize = 3;
            let unit_shards = (0..AMOUNT)
                .map(|_| TUnitShardItem {
                    id: next_id(),
                    owner: 0,
                    unit: TBaseUnit::get_random_for_lootbox().name,
                    count: rng().gen_range(3..7),
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
            TTrade::open_lootbox(user.id, bundle)?;
        }
    }
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
