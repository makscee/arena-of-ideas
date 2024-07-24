use ability::TAbility;
use base_unit::TBaseUnit;
use house::THouse;
use representation::TRepresentation;
use spacetimedb::TableType;
use status::TStatus;

use super::*;

fn replace_assets(
    global_settings: GlobalSettings,
    representations: Vec<TRepresentation>,
    base_units: Vec<TBaseUnit>,
    houses: Vec<THouse>,
    abilities: Vec<TAbility>,
    statuses: Vec<TStatus>,
) -> Result<(), String> {
    global_settings.replace();
    for r in TRepresentation::iter() {
        r.delete();
    }
    for r in representations {
        TRepresentation::insert(r)?;
    }
    for unit in TBaseUnit::iter() {
        unit.delete();
    }
    for unit in base_units {
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

#[spacetimedb(reducer)]
fn upload_assets(
    ctx: ReducerContext,
    global_settings: GlobalSettings,
    representations: Vec<TRepresentation>,
    base_units: Vec<TBaseUnit>,
    houses: Vec<THouse>,
    abilities: Vec<TAbility>,
    statuses: Vec<TStatus>,
) -> Result<(), String> {
    replace_assets(
        global_settings,
        representations,
        base_units,
        houses,
        abilities,
        statuses,
    )
}

#[spacetimedb(reducer)]
fn upload_game_archive(
    ctx: ReducerContext,
    global_settings: GlobalSettings,
    global_data: GlobalData,
    users: Vec<TUser>,
    base_units: Vec<TBaseUnit>,
    houses: Vec<THouse>,
    abilities: Vec<TAbility>,
    statuses: Vec<TStatus>,
    representations: Vec<TRepresentation>,
    arena_runs: Vec<TArenaRun>,
    arena_runs_archive: Vec<TArenaRunArchive>,
    arena_leaderboard: Vec<TArenaLeaderboard>,
    teams: Vec<TTeam>,
    battles: Vec<TBattle>,
) -> Result<(), String> {
    replace_assets(
        global_settings,
        representations,
        base_units,
        houses,
        abilities,
        statuses,
    )?;
    GlobalData::insert(global_data)?;
    for d in TUser::iter() {
        d.delete();
    }
    for d in users {
        TUser::insert(d)?;
    }
    for d in TArenaRun::iter() {
        d.delete();
    }
    for d in arena_runs {
        TArenaRun::insert(d)?;
    }
    for d in TArenaRunArchive::iter() {
        d.delete();
    }
    for d in arena_runs_archive {
        TArenaRunArchive::insert(d)?;
    }
    for d in TArenaLeaderboard::iter() {
        d.delete();
    }
    for d in arena_leaderboard {
        TArenaLeaderboard::insert(d);
    }
    for d in TTeam::iter() {
        d.delete();
    }
    for d in teams {
        TTeam::insert(d)?;
    }
    for d in TBattle::iter() {
        d.delete();
    }
    for d in battles {
        TBattle::insert(d)?;
    }

    Ok(())
}
