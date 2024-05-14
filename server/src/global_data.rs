use spacetimedb::Timestamp;

use super::*;

#[spacetimedb(table)]
pub struct GlobalData {
    #[unique]
    always_zero: u32,
    next_id: u64,
    pub game_version: String,
    pub last_sync: Timestamp,
}

const VERSION: &str = env!("CARGO_PKG_VERSION");
impl GlobalData {
    pub fn init() -> Result<(), String> {
        GlobalData::insert(GlobalData {
            always_zero: 0,
            next_id: 1,
            game_version: VERSION.to_owned(),
            last_sync: Timestamp::UNIX_EPOCH,
        })?;
        Ok(())
    }

    pub fn next_id() -> u64 {
        let mut gd = GlobalData::filter_by_always_zero(&0).unwrap();
        let id = gd.next_id;
        gd.next_id += 1;
        GlobalData::update_by_always_zero(&0, gd);
        id
    }

    pub fn get() -> Self {
        GlobalData::filter_by_always_zero(&0).unwrap()
    }
}

#[spacetimedb(reducer)]
fn upload_units(ctx: ReducerContext, units: Vec<TableUnit>) -> Result<(), String> {
    UserRight::UnitSync.check(&ctx.sender)?;
    for unit in units {
        TableUnit::delete_by_name(&unit.name);
        TableUnit::insert(unit)?;
    }
    let mut gd = GlobalData::filter_by_always_zero(&0).unwrap();
    gd.last_sync = Timestamp::now();
    GlobalData::update_by_always_zero(&0, gd);
    Ok(())
}

#[spacetimedb(reducer)]
fn sync_data(
    ctx: ReducerContext,
    houses: Vec<House>,
    abilities: Vec<Ability>,
    statuses: Vec<Statuses>,
    summons: Vec<Summon>,
    units: Vec<TableUnit>,
    vfxs: Vec<Vfx>,
) -> Result<(), String> {
    UserRight::UnitSync.check(&ctx.sender)?;
    for house in House::iter() {
        House::delete_by_name(&house.name);
    }
    for house in houses {
        House::insert(house)?;
    }
    for ability in Ability::iter() {
        Ability::delete_by_name(&ability.name);
    }
    for ability in abilities {
        Ability::insert(ability)?;
    }
    for status in Statuses::iter() {
        Statuses::delete_by_name(&status.name);
    }
    for status in statuses {
        Statuses::insert(status)?;
    }
    for summon in Summon::iter() {
        Summon::delete_by_name(&summon.name);
    }
    for summon in summons {
        Summon::insert(summon)?;
    }
    for unit in TableUnit::iter() {
        TableUnit::delete_by_name(&unit.name);
    }
    for unit in units {
        TableUnit::insert(unit)?;
    }
    for vfx in Vfx::iter() {
        Vfx::delete_by_name(&vfx.name);
    }
    for vfx in vfxs {
        Vfx::insert(vfx)?;
    }
    let mut gd = GlobalData::filter_by_always_zero(&0).unwrap();
    gd.last_sync = Timestamp::now();
    GlobalData::update_by_always_zero(&0, gd);
    Ok(())
}

#[spacetimedb(reducer)]
fn migrate_data(
    ctx: ReducerContext,
    arena_archive: Vec<ArenaArchive>,
    arena_pool: Vec<ArenaPool>,
    users: Vec<User>,
) -> Result<(), String> {
    UserRight::UnitSync.check(&ctx.sender)?;
    let mut max_id = 0;
    for v in arena_archive {
        max_id = max_id.max(v.id);
        ArenaArchive::insert(v)?;
    }
    for v in arena_pool {
        max_id = max_id.max(v.id);
        ArenaPool::insert(v)?;
    }
    for v in User::iter() {
        User::delete_by_id(&v.id);
    }
    for v in users {
        max_id = max_id.max(v.id);
        User::insert(v)?;
    }
    let mut gd = GlobalData::get();
    gd.next_id = max_id + 1;
    GlobalData::update_by_always_zero(&0, gd);
    Ok(())
}
