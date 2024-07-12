use ability::TAbility;
use base_unit::TBaseUnit;
use house::THouse;
use representation::TRepresentation;
use spacetimedb::TableType;
use status::TStatus;

use super::*;

#[spacetimedb(reducer)]
fn sync_all_assets(
    ctx: ReducerContext,
    gs: GlobalSettings,
    representations: Vec<TRepresentation>,
    units: Vec<TBaseUnit>,
    houses: Vec<THouse>,
    abilities: Vec<TAbility>,
    statuses: Vec<TStatus>,
) -> Result<(), String> {
    gs.replace();
    for r in TRepresentation::iter() {
        r.delete();
    }
    for r in representations {
        TRepresentation::insert(r)?;
    }
    for unit in TBaseUnit::iter() {
        unit.delete();
    }
    for unit in units {
        TBaseUnit::insert(unit)?;
    }
    for house in THouse::iter() {
        house.delete();
    }
    for house in houses {
        THouse::insert(house)?;
    }
    for status in TStatus::iter() {
        status.delete();
    }
    for status in statuses {
        TStatus::insert(status)?;
    }
    for ability in TAbility::iter() {
        ability.delete();
    }
    for ability in abilities {
        TAbility::insert(ability)?;
    }
    GlobalData::register_sync();
    Ok(())
}
