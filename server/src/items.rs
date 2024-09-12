use super::*;

#[derive(SpacetimeType, Default)]
pub struct ItemBundle {
    pub units: Vec<u64>,
    pub unit_shards: Vec<u64>,
    pub lootboxes: Vec<u64>,
}

#[spacetimedb(table(public))]
pub struct TUnitItem {
    #[primarykey]
    pub id: u64,
    pub owner: u64,
    pub unit: FusedUnit,
}

#[spacetimedb(table(public))]
pub struct TUnitShardItem {
    #[primarykey]
    pub id: u64,
    pub owner: u64,
    pub unit: String,
    pub count: u32,
}

#[spacetimedb(table(public))]
pub struct TLootboxItem {
    #[primarykey]
    pub id: u64,
    pub owner: u64,
    pub kind: LootboxKind,
    pub count: u32,
}

#[derive(SpacetimeType, Copy, Clone)]
pub enum LootboxKind {
    Regular,
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

impl TUnitItem {
    fn from_fused_unit(owner: u64, unit: FusedUnit) -> Self {
        Self {
            id: next_id(),
            owner,
            unit,
        }
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
            let mut shard = TUnitShardItem::filter_by_id(&id)
                .with_context_str(|| format!("UnitShard {id} not found"))?;
            shard.owner = owner;
            TUnitShardItem::update_by_id(&shard.id.clone(), shard);
        }
        for id in self.lootboxes {
            let mut lootbox = TLootboxItem::filter_by_id(&id)
                .with_context_str(|| format!("UnitShard {id} not found"))?;
            lootbox.owner = owner;
            TLootboxItem::update_by_id(&lootbox.id.clone(), lootbox);
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
    TUnitShardItem::insert(item)?;
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
                    TUnitShardItem::insert(s);
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
