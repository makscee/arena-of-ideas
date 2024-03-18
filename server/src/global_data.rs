use spacetimedb::Timestamp;

use super::*;

#[spacetimedb(table)]
pub struct GlobalData {
    #[unique]
    always_zero: u32,
    pub game_version: String,
    pub last_sync: Timestamp,
}

const VERSION: &str = env!("CARGO_PKG_VERSION");
impl GlobalData {
    pub fn init() -> Result<(), String> {
        GlobalData::insert(GlobalData {
            always_zero: 0,
            game_version: VERSION.to_owned(),
            last_sync: Timestamp::UNIX_EPOCH,
        })?;
        Ok(())
    }
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
