use ability::TAbility;
use auction::TAuction;
use base_unit::TBaseUnit;
use house::THouse;

use status::TStatus;

use super::*;

#[derive(SpacetimeType, Default)]
struct GameData {
    global_data: Vec<GlobalData>,
    global_settings: Vec<GlobalSettings>,
    ability: Vec<TAbility>,
    arena_leaderboard: Vec<TArenaLeaderboard>,
    arena_run: Vec<TArenaRun>,
    arena_run_archive: Vec<TArenaRunArchive>,
    auction: Vec<TAuction>,
    base_unit: Vec<TBaseUnit>,
    battle: Vec<TBattle>,
    daily_state: Vec<TDailyState>,
    house: Vec<THouse>,
    lootbox_item: Vec<TLootboxItem>,
    meta_shop: Vec<TMetaShop>,
    quest: Vec<TQuest>,
    rainbow_shard_item: Vec<TRainbowShardItem>,
    status: Vec<TStatus>,
    team: Vec<TTeam>,
    trade: Vec<TTrade>,
    unit_balance: Vec<TUnitBalance>,
    unit_item: Vec<TUnitItem>,
    unit_shard_item: Vec<TUnitShardItem>,
    user: Vec<TUser>,
    user_stats: Vec<TUserStats>,
    user_game_stats: Vec<TUserGameStats>,
    wallet: Vec<TWallet>,
    incubator: Vec<TIncubator>,
    incubator_vote: Vec<TIncubatorVote>,
    incubator_favorite: Vec<TIncubatorFavorite>,
}

fn replace<E: TableType>(data: Vec<E>) {
    if data.is_empty() {
        return;
    }
    for r in E::iter() {
        r.delete();
    }
    for r in data {
        E::insert(r);
    }
}

fn replace_assets(data: GameData) -> Result<(), String> {
    let GameData {
        mut global_data,
        mut global_settings,
        ability,
        arena_leaderboard,
        arena_run,
        arena_run_archive,
        auction,
        base_unit,
        battle,
        daily_state,
        house,
        lootbox_item,
        meta_shop,
        quest,
        rainbow_shard_item,
        status,
        team,
        trade,
        unit_balance,
        unit_item,
        unit_shard_item,
        user,
        wallet,
        incubator,
        incubator_vote,
        incubator_favorite,
        user_stats,
        user_game_stats,
    } = data;
    if !global_settings.is_empty() {
        global_settings.remove(0).replace();
    }
    if !global_data.is_empty() {
        GlobalData::delete_by_always_zero(&0);
        GlobalData::insert(global_data.remove(0)).unwrap();
    }
    replace(ability);
    replace(arena_leaderboard);
    replace(arena_run);
    replace(arena_run_archive);
    replace(auction);
    replace(base_unit);
    replace(battle);
    replace(daily_state);
    replace(house);
    replace(lootbox_item);
    replace(meta_shop);
    replace(quest);
    replace(rainbow_shard_item);
    replace(status);
    replace(team);
    replace(trade);
    replace(unit_balance);
    replace(unit_item);
    replace(unit_shard_item);
    replace(user);
    replace(wallet);
    replace(incubator);
    replace(incubator_vote);
    replace(incubator_favorite);
    replace(user_stats);
    replace(user_game_stats);

    let ghost = || FusedUnit::from_base_name(GlobalSettings::get().ghost_unit, next_id()).unwrap();
    let enemies = [
        TTeam::new(0, TeamPool::Enemy).units(vec![ghost()]).save(),
        TTeam::new(0, TeamPool::Enemy)
            .units(vec![ghost(), ghost()])
            .save(),
        TTeam::new(0, TeamPool::Enemy)
            .units(vec![ghost(), ghost(), ghost()])
            .save(),
    ]
    .into();
    GlobalData::set_initial_enemies(enemies);
    if GlobalData::get().last_sync.eq(&Timestamp::UNIX_EPOCH) {
        daily_timer_init();
    }
    GlobalData::register_sync();
    Ok(())
}

#[spacetimedb(reducer)]
fn upload_assets(
    ctx: ReducerContext,
    global_settings: GlobalSettings,
    base_unit: Vec<TBaseUnit>,
    house: Vec<THouse>,
    ability: Vec<TAbility>,
    status: Vec<TStatus>,
) -> Result<(), String> {
    ctx.is_admin()?;
    replace_assets(GameData {
        global_settings: vec![global_settings],
        ability,
        base_unit,
        house,
        status,
        ..default()
    })
}

#[spacetimedb(reducer)]
fn upload_game_data(ctx: ReducerContext, next_id: u64, data: GameData) -> Result<(), String> {
    ctx.is_admin()?;
    if next_id > 0 {
        GlobalData::set_next_id(next_id);
    }
    replace_assets(data)?;
    Ok(())
}