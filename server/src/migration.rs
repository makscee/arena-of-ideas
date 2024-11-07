use ability::ability;
use arena_leaderboard::arena_leaderboard;
use auction::auction;
use base_unit::base_unit;
use battle::battle;
use daily_state::daily_state;
use global_data::global_data;
use global_event::global_event;
use house::house;
use incubator::incubator;
use meta_shop::meta_shop;
use player::player;
use player_stats::player_stats;
use quest::quest;
use spacetimedb::Table;
use status::status;
use team::team;
use trade::trade;
use unit_balance::unit_balance;
use wallet::wallet;

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
    player: Vec<TPlayer>,
    player_stats: Vec<TPlayerStats>,
    player_game_stats: Vec<TPlayerGameStats>,
    wallet: Vec<TWallet>,
    incubator: Vec<TIncubator>,
    incubator_vote: Vec<TIncubatorVote>,
    incubator_favorite: Vec<TIncubatorFavorite>,
    global_event: Vec<TGlobalEvent>,
}

fn replace_assets(ctx: &ReducerContext, data: GameData) -> Result<(), String> {
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
        player,
        wallet,
        incubator,
        incubator_vote,
        incubator_favorite,
        player_stats,
        player_game_stats,
        global_event,
    } = data;
    if !global_settings.is_empty() {
        global_settings.remove(0).replace(ctx);
    }
    if !global_data.is_empty() {
        ctx.db.global_data().always_zero().delete(0);
        ctx.db.global_data().insert(global_data.remove(0));
    }
    if !ability.is_empty() {
        for d in ctx.db.ability().iter() {
            ctx.db.ability().delete(d);
        }
        for d in ability {
            ctx.db.ability().insert(d);
        }
    }
    if !arena_leaderboard.is_empty() {
        for d in ctx.db.arena_leaderboard().iter() {
            ctx.db.arena_leaderboard().delete(d);
        }
        for d in arena_leaderboard {
            ctx.db.arena_leaderboard().insert(d);
        }
    }
    if !arena_run.is_empty() {
        for d in ctx.db.arena_run().iter() {
            ctx.db.arena_run().delete(d);
        }
        for d in arena_run {
            ctx.db.arena_run().insert(d);
        }
    }
    if !arena_run_archive.is_empty() {
        for d in ctx.db.arena_run_archive().iter() {
            ctx.db.arena_run_archive().delete(d);
        }
        for d in arena_run_archive {
            ctx.db.arena_run_archive().insert(d);
        }
    }
    if !auction.is_empty() {
        for d in ctx.db.auction().iter() {
            ctx.db.auction().delete(d);
        }
        for d in auction {
            ctx.db.auction().insert(d);
        }
    }
    if !base_unit.is_empty() {
        for d in ctx.db.base_unit().iter() {
            ctx.db.base_unit().delete(d);
        }
        for d in base_unit {
            ctx.db.base_unit().insert(d);
        }
    }
    if !battle.is_empty() {
        for d in ctx.db.battle().iter() {
            ctx.db.battle().delete(d);
        }
        for d in battle {
            ctx.db.battle().insert(d);
        }
    }
    if !daily_state.is_empty() {
        for d in ctx.db.daily_state().iter() {
            ctx.db.daily_state().delete(d);
        }
        for d in daily_state {
            ctx.db.daily_state().insert(d);
        }
    }
    if !house.is_empty() {
        for d in ctx.db.house().iter() {
            ctx.db.house().delete(d);
        }
        for d in house {
            ctx.db.house().insert(d);
        }
    }
    if !lootbox_item.is_empty() {
        for d in ctx.db.lootbox_item().iter() {
            ctx.db.lootbox_item().delete(d);
        }
        for d in lootbox_item {
            ctx.db.lootbox_item().insert(d);
        }
    }
    if !meta_shop.is_empty() {
        for d in ctx.db.meta_shop().iter() {
            ctx.db.meta_shop().delete(d);
        }
        for d in meta_shop {
            ctx.db.meta_shop().insert(d);
        }
    }
    if !quest.is_empty() {
        for d in ctx.db.quest().iter() {
            ctx.db.quest().delete(d);
        }
        for d in quest {
            ctx.db.quest().insert(d);
        }
    }
    if !rainbow_shard_item.is_empty() {
        for d in ctx.db.rainbow_shard_item().iter() {
            ctx.db.rainbow_shard_item().delete(d);
        }
        for d in rainbow_shard_item {
            ctx.db.rainbow_shard_item().insert(d);
        }
    }
    if !status.is_empty() {
        for d in ctx.db.status().iter() {
            ctx.db.status().delete(d);
        }
        for d in status {
            ctx.db.status().insert(d);
        }
    }
    if !team.is_empty() {
        for d in ctx.db.team().iter() {
            ctx.db.team().delete(d);
        }
        for d in team {
            ctx.db.team().insert(d);
        }
    }
    if !trade.is_empty() {
        for d in ctx.db.trade().iter() {
            ctx.db.trade().delete(d);
        }
        for d in trade {
            ctx.db.trade().insert(d);
        }
    }
    if !unit_balance.is_empty() {
        for d in ctx.db.unit_balance().iter() {
            ctx.db.unit_balance().delete(d);
        }
        for d in unit_balance {
            ctx.db.unit_balance().insert(d);
        }
    }
    if !unit_item.is_empty() {
        for d in ctx.db.unit_item().iter() {
            ctx.db.unit_item().delete(d);
        }
        for d in unit_item {
            ctx.db.unit_item().insert(d);
        }
    }
    if !unit_shard_item.is_empty() {
        for d in ctx.db.unit_shard_item().iter() {
            ctx.db.unit_shard_item().delete(d);
        }
        for d in unit_shard_item {
            ctx.db.unit_shard_item().insert(d);
        }
    }
    if !player.is_empty() {
        for d in ctx.db.player().iter() {
            ctx.db.player().delete(d);
        }
        for d in player {
            ctx.db.player().insert(d);
        }
    }
    if !wallet.is_empty() {
        for d in ctx.db.wallet().iter() {
            ctx.db.wallet().delete(d);
        }
        for d in wallet {
            ctx.db.wallet().insert(d);
        }
    }
    if !incubator.is_empty() {
        for d in ctx.db.incubator().iter() {
            ctx.db.incubator().delete(d);
        }
        for d in incubator {
            ctx.db.incubator().insert(d);
        }
    }
    if !incubator_vote.is_empty() {
        for d in ctx.db.incubator_vote().iter() {
            ctx.db.incubator_vote().delete(d);
        }
        for d in incubator_vote {
            ctx.db.incubator_vote().insert(d);
        }
    }
    if !incubator_favorite.is_empty() {
        for d in ctx.db.incubator_favorite().iter() {
            ctx.db.incubator_favorite().delete(d);
        }
        for d in incubator_favorite {
            ctx.db.incubator_favorite().insert(d);
        }
    }
    if !player_stats.is_empty() {
        for d in ctx.db.player_stats().iter() {
            ctx.db.player_stats().delete(d);
        }
        for d in player_stats {
            ctx.db.player_stats().insert(d);
        }
    }
    if !player_game_stats.is_empty() {
        for d in ctx.db.player_game_stats().iter() {
            ctx.db.player_game_stats().delete(d);
        }
        for d in player_game_stats {
            ctx.db.player_game_stats().insert(d);
        }
    }
    if !global_event.is_empty() {
        for d in ctx.db.global_event().iter() {
            ctx.db.global_event().delete(d);
        }
        for d in global_event {
            ctx.db.global_event().insert(d);
        }
    }

    let ghost = || {
        FusedUnit::from_base_name(ctx, GlobalSettings::get(ctx).ghost_unit, next_id(ctx)).unwrap()
    };
    let enemies = [
        TTeam::new(ctx, 0, TeamPool::Enemy)
            .units(vec![ghost()])
            .save(ctx),
        TTeam::new(ctx, 0, TeamPool::Enemy)
            .units(vec![ghost(), ghost()])
            .save(ctx),
        TTeam::new(ctx, 0, TeamPool::Enemy)
            .units(vec![ghost(), ghost(), ghost()])
            .save(ctx),
    ]
    .into();
    GlobalData::set_initial_enemies(ctx, enemies);
    if GlobalData::get(ctx).last_sync.eq(&Timestamp::UNIX_EPOCH) {
        daily_timer_init(ctx);
    }
    GlobalData::register_sync(ctx);
    Ok(())
}

#[spacetimedb::reducer]
fn upload_assets(
    ctx: &ReducerContext,
    global_settings: GlobalSettings,
    base_unit: Vec<TBaseUnit>,
    house: Vec<THouse>,
    ability: Vec<TAbility>,
    status: Vec<TStatus>,
) -> Result<(), String> {
    ctx.is_admin()?;
    replace_assets(
        ctx,
        GameData {
            global_settings: vec![global_settings],
            ability,
            base_unit,
            house,
            status,
            ..default()
        },
    )
}

#[spacetimedb::reducer]
fn upload_game_data(ctx: &ReducerContext, next_id: u64, data: GameData) -> Result<(), String> {
    ctx.is_admin()?;
    if next_id > 0 {
        GlobalData::set_next_id(ctx, next_id);
    }
    replace_assets(ctx, data)?;
    Ok(())
}
