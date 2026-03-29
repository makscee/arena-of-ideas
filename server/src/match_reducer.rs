use spacetimedb::{ReducerContext, Table};

#[allow(unused_imports)]
use crate::{
    arena_state, floor_boss, floor_pool_team, game_match, player, unit, ArenaState, ContentStatus,
    FloorBoss, FloorPoolTeam, GameMatch, MatchState, TeamSlot, Unit,
};

const STARTING_GOLD: i32 = 7;
const STARTING_LIVES: i32 = 3;
const TEAM_SIZE: usize = 5;
const REROLL_COST: i32 = 1;
const GOLD_REWARD: i32 = 3;

fn shop_size(floor: u8) -> usize {
    match floor {
        1..=2 => 3,
        3..=4 => 4,
        _ => 5,
    }
}

fn tier_cost(tier: u8) -> i32 {
    tier as i32
}

fn sell_value(_tier: u8) -> i32 {
    1
}

fn get_last_floor(ctx: &ReducerContext) -> u8 {
    ctx.db
        .arena_state()
        .always_zero()
        .find(0)
        .map(|a| a.last_floor)
        .unwrap_or(1)
}

// ===== Match Lifecycle =====

#[spacetimedb::reducer]
pub fn match_start(ctx: &ReducerContext) -> Result<(), String> {
    if ctx.db.player().identity().find(ctx.sender()).is_none() {
        return Err("Player not registered".to_string());
    }

    for m in ctx.db.game_match().iter() {
        if m.player == ctx.sender() {
            return Err("Player already has an active match".to_string());
        }
    }

    let offers = generate_shop_offers(ctx, shop_size(1));

    ctx.db.game_match().insert(GameMatch {
        id: 0,
        player: ctx.sender(),
        floor: 1,
        gold: STARTING_GOLD,
        lives: STARTING_LIVES,
        state: MatchState::Shop,
        team: Vec::new(),
        shop_offers: offers,
        pending_battle_id: 0,
        created_at: ctx.timestamp,
    });

    Ok(())
}

#[spacetimedb::reducer]
pub fn match_abandon(ctx: &ReducerContext) -> Result<(), String> {
    let game_match = find_player_match(ctx)?;
    ctx.db.game_match().id().delete(game_match.id);
    Ok(())
}

// ===== Shop Actions =====

#[spacetimedb::reducer]
pub fn match_shop_buy(ctx: &ReducerContext, shop_index: u32) -> Result<(), String> {
    let mut game_match = find_player_match(ctx)?;
    require_state(&game_match, &MatchState::Shop)?;

    let idx = shop_index as usize;
    if idx >= game_match.shop_offers.len() {
        return Err("Invalid shop index".to_string());
    }

    let unit_id = game_match.shop_offers[idx];
    if unit_id == 0 {
        return Err("Shop slot is empty".to_string());
    }

    let unit = ctx
        .db
        .unit()
        .id()
        .find(unit_id)
        .ok_or("Unit not found")?;

    let cost = tier_cost(unit.tier);
    if game_match.gold < cost {
        return Err(format!(
            "Not enough gold (have {}, need {})",
            game_match.gold, cost
        ));
    }

    // Check for stacking (duplicate unit)
    let mut stacked = false;
    for slot in &mut game_match.team {
        if slot.unit_id == unit_id && !slot.is_fused {
            slot.copies += 1;
            slot.bonus_hp += 1;
            slot.bonus_pwr += 1;
            stacked = true;
            break;
        }
    }

    if !stacked {
        if game_match.team.len() >= TEAM_SIZE {
            return Err("Team is full".to_string());
        }
        game_match.team.push(TeamSlot {
            unit_id,
            copies: 1,
            bonus_hp: 0,
            bonus_pwr: 0,
            is_fused: false,
            fused_trigger: String::new(),
            fused_abilities: Vec::new(),
            fused_tier: 0,
        });
    }

    game_match.gold -= cost;
    game_match.shop_offers[idx] = 0;

    ctx.db.game_match().id().update(game_match);
    Ok(())
}

#[spacetimedb::reducer]
pub fn match_sell_unit(ctx: &ReducerContext, slot_index: u32) -> Result<(), String> {
    let mut game_match = find_player_match(ctx)?;
    require_state(&game_match, &MatchState::Shop)?;

    let idx = slot_index as usize;
    if idx >= game_match.team.len() {
        return Err("Invalid slot index".to_string());
    }

    let slot = &game_match.team[idx];
    let tier = if slot.is_fused {
        slot.fused_tier
    } else {
        ctx.db.unit().id().find(slot.unit_id).map(|u| u.tier).unwrap_or(1)
    };

    game_match.gold += sell_value(tier);
    game_match.team.remove(idx);

    ctx.db.game_match().id().update(game_match);
    Ok(())
}

#[spacetimedb::reducer]
pub fn match_shop_reroll(ctx: &ReducerContext) -> Result<(), String> {
    let mut game_match = find_player_match(ctx)?;
    require_state(&game_match, &MatchState::Shop)?;

    if game_match.gold < REROLL_COST {
        return Err("Not enough gold to reroll".to_string());
    }

    game_match.gold -= REROLL_COST;
    game_match.shop_offers = generate_shop_offers(ctx, shop_size(game_match.floor));

    ctx.db.game_match().id().update(game_match);
    Ok(())
}

#[spacetimedb::reducer]
pub fn match_move_unit(ctx: &ReducerContext, from_slot: u32, to_slot: u32) -> Result<(), String> {
    let mut game_match = find_player_match(ctx)?;
    require_state(&game_match, &MatchState::Shop)?;

    let from = from_slot as usize;
    let to = to_slot as usize;

    if from >= game_match.team.len() || to >= game_match.team.len() {
        return Err("Invalid slot index".to_string());
    }

    game_match.team.swap(from, to);
    ctx.db.game_match().id().update(game_match);
    Ok(())
}

// ===== Battle Actions =====

/// Start a battle. Type determined by floor vs last_floor.
#[spacetimedb::reducer]
pub fn match_start_battle(ctx: &ReducerContext) -> Result<(), String> {
    let mut game_match = find_player_match(ctx)?;
    require_state(&game_match, &MatchState::Shop)?;

    let last_floor = get_last_floor(ctx);
    let floor = game_match.floor;

    // Snapshot team to JSON for floor pool
    let team_snapshot = serde_json::to_string(&game_match.team)
        .map_err(|e| format!("Failed to snapshot team: {}", e))?;

    if floor < last_floor {
        // Regular battle: add team to pool, pick random opponent
        ctx.db.floor_pool_team().insert(FloorPoolTeam {
            id: 0,
            floor,
            player: ctx.sender(),
            team_snapshot,
            created_at: ctx.timestamp,
        });

        game_match.state = MatchState::RegularBattle;
    } else if floor == last_floor {
        // Boss battle
        ctx.db.floor_pool_team().insert(FloorPoolTeam {
            id: 0,
            floor,
            player: ctx.sender(),
            team_snapshot,
            created_at: ctx.timestamp,
        });

        game_match.state = MatchState::BossBattle;
    } else {
        // Champion battle (floor > last_floor)
        game_match.state = MatchState::ChampionBattle;
    }

    ctx.db.game_match().id().update(game_match);
    Ok(())
}

/// Submit battle result. Handles all battle types.
#[spacetimedb::reducer]
pub fn match_submit_result(ctx: &ReducerContext, won: bool) -> Result<(), String> {
    let mut game_match = find_player_match(ctx)?;

    let battle_state = game_match.state.clone();

    match battle_state {
        MatchState::RegularBattle => {
            // Win or lose: advance floor, get gold
            game_match.floor += 1;
            game_match.gold += GOLD_REWARD;

            if !won {
                game_match.lives -= 1;
                if game_match.lives <= 0 {
                    // Game over
                    ctx.db.game_match().id().delete(game_match.id);
                    return Ok(());
                }
            }

            // Refresh shop for next floor
            game_match.shop_offers = generate_shop_offers(ctx, shop_size(game_match.floor));
            game_match.state = MatchState::Shop;
            ctx.db.game_match().id().update(game_match);
        }

        MatchState::BossBattle => {
            if won {
                // Push frontier: become boss, advance last_floor
                let team_snapshot = serde_json::to_string(&game_match.team).unwrap_or_default();
                let floor = game_match.floor;

                // Set floor boss
                let mut found_boss = false;
                for mut boss in ctx.db.floor_boss().iter() {
                    if boss.floor == floor {
                        boss.team_snapshot = team_snapshot.clone();
                        boss.player = ctx.sender();
                        ctx.db.floor_boss().id().update(boss);
                        found_boss = true;
                        break;
                    }
                }
                if !found_boss {
                    ctx.db.floor_boss().insert(FloorBoss {
                        id: 0,
                        floor,
                        player: ctx.sender(),
                        team_snapshot,
                        created_at: ctx.timestamp,
                    });
                }

                // Advance frontier
                if let Some(mut arena) = ctx.db.arena_state().always_zero().find(0) {
                    if floor >= arena.last_floor {
                        arena.last_floor = floor + 1;
                        ctx.db.arena_state().always_zero().update(arena);
                    }
                }

                // Advance floor, get gold, continue
                game_match.floor += 1;
                game_match.gold += GOLD_REWARD;
                game_match.shop_offers =
                    generate_shop_offers(ctx, shop_size(game_match.floor));
                game_match.state = MatchState::Shop;
                ctx.db.game_match().id().update(game_match);
            } else {
                // Game over — boss loss is instant death
                ctx.db.game_match().id().delete(game_match.id);
            }
        }

        MatchState::ChampionBattle => {
            if won {
                // Victory! Push frontier, become new boss
                let team_snapshot = serde_json::to_string(&game_match.team).unwrap_or_default();
                let floor = game_match.floor;

                // Update or create floor boss
                let mut found_boss = false;
                for mut boss in ctx.db.floor_boss().iter() {
                    if boss.floor == floor {
                        boss.team_snapshot = team_snapshot.clone();
                        boss.player = ctx.sender();
                        ctx.db.floor_boss().id().update(boss);
                        found_boss = true;
                        break;
                    }
                }
                if !found_boss {
                    ctx.db.floor_boss().insert(FloorBoss {
                        id: 0,
                        floor,
                        player: ctx.sender(),
                        team_snapshot,
                        created_at: ctx.timestamp,
                    });
                }

                // Advance frontier
                if let Some(mut arena) = ctx.db.arena_state().always_zero().find(0) {
                    if floor >= arena.last_floor {
                        arena.last_floor = floor + 1;
                        ctx.db.arena_state().always_zero().update(arena);
                    }
                }

                // Match over — victory
                ctx.db.game_match().id().delete(game_match.id);
            } else {
                // Game over
                ctx.db.game_match().id().delete(game_match.id);
            }
        }

        _ => {
            return Err("Not in a battle state".to_string());
        }
    }

    Ok(())
}

// ===== Fusion =====

#[spacetimedb::reducer]
pub fn match_fuse_units(
    ctx: &ReducerContext,
    slot_a: u32,
    slot_b: u32,
    trigger_from_a: bool,
    chosen_abilities: Vec<u64>,
) -> Result<(), String> {
    let mut game_match = find_player_match(ctx)?;
    require_state(&game_match, &MatchState::Shop)?;

    let a = slot_a as usize;
    let b = slot_b as usize;

    if a >= game_match.team.len() || b >= game_match.team.len() || a == b {
        return Err("Invalid slot indices".to_string());
    }

    let slot_a_data = &game_match.team[a];
    let slot_b_data = &game_match.team[b];

    if slot_a_data.copies < 3 {
        return Err("Unit A needs 3 copies to fuse".to_string());
    }
    if slot_a_data.is_fused || slot_b_data.is_fused {
        return Err("Cannot fuse already-fused units".to_string());
    }

    let unit_a = ctx.db.unit().id().find(slot_a_data.unit_id).ok_or("Unit A not found")?;
    let unit_b = ctx.db.unit().id().find(slot_b_data.unit_id).ok_or("Unit B not found")?;

    let result_tier = unit_a.tier.max(unit_b.tier) + 1;
    if result_tier > 5 {
        return Err("Fusion would exceed max tier 5".to_string());
    }

    let abilities_a = &unit_a.abilities;
    let abilities_b = &unit_b.abilities;
    let result_ability_count = abilities_a.len().min(abilities_b.len()) + 1;

    if chosen_abilities.len() != result_ability_count {
        return Err(format!(
            "Must choose exactly {} abilities, got {}",
            result_ability_count,
            chosen_abilities.len()
        ));
    }

    let combined: Vec<u64> = abilities_a.iter().chain(abilities_b.iter()).copied().collect();
    for &chosen in &chosen_abilities {
        if !combined.contains(&chosen) {
            return Err(format!("Ability {} not from either parent", chosen));
        }
    }

    let trigger_str = if trigger_from_a {
        format!("{:?}", unit_a.trigger)
    } else {
        format!("{:?}", unit_b.trigger)
    };

    let (remove_first, remove_second) = if a > b { (a, b) } else { (b, a) };
    game_match.team.remove(remove_first);
    let fuse_idx = if a > b {
        remove_second
    } else {
        a.min(game_match.team.len())
    };

    if fuse_idx < game_match.team.len() {
        let slot = &mut game_match.team[fuse_idx];
        slot.is_fused = true;
        slot.fused_trigger = trigger_str;
        slot.fused_abilities = chosen_abilities;
        slot.fused_tier = result_tier;
        slot.copies = 1;
        // Fused stats: max(a,b) + min(a,b)/2
        // The slot already carries unit_a's base stats, so bonus = fused - base_a
        let fused_hp = unit_a.hp.max(unit_b.hp) + unit_a.hp.min(unit_b.hp) / 2;
        let fused_pwr = unit_a.pwr.max(unit_b.pwr) + unit_a.pwr.min(unit_b.pwr) / 2;
        slot.bonus_hp = fused_hp - unit_a.hp;
        slot.bonus_pwr = fused_pwr - unit_a.pwr;
    }

    ctx.db.game_match().id().update(game_match);
    Ok(())
}

#[spacetimedb::reducer]
pub fn match_feed_unit(ctx: &ReducerContext, fused_slot: u32, donor_slot: u32) -> Result<(), String> {
    let mut game_match = find_player_match(ctx)?;
    require_state(&game_match, &MatchState::Shop)?;

    let f = fused_slot as usize;
    let d = donor_slot as usize;

    if f >= game_match.team.len() || d >= game_match.team.len() || f == d {
        return Err("Invalid slot indices".to_string());
    }

    if !game_match.team[f].is_fused {
        return Err("Target must be a fused unit".to_string());
    }

    let donor_unit = ctx.db.unit().id().find(game_match.team[d].unit_id).ok_or("Donor not found")?;

    for &donor_ability in &donor_unit.abilities {
        if !game_match.team[f].fused_abilities.contains(&donor_ability) {
            return Err(format!("Donor ability {} not in fused unit", donor_ability));
        }
    }

    game_match.team[f].bonus_hp += 2;
    game_match.team[f].bonus_pwr += 1;
    game_match.team.remove(d);

    ctx.db.game_match().id().update(game_match);
    Ok(())
}

// ===== Helpers =====

fn find_player_match(ctx: &ReducerContext) -> Result<GameMatch, String> {
    for m in ctx.db.game_match().iter() {
        if m.player == ctx.sender() {
            return Ok(m);
        }
    }
    Err("No active match found".to_string())
}

fn require_state(game_match: &GameMatch, expected: &MatchState) -> Result<(), String> {
    if game_match.state != *expected {
        Err(format!(
            "Wrong state: expected {:?}, got {:?}",
            expected, game_match.state
        ))
    } else {
        Ok(())
    }
}

/// Simple hash-based PRNG (no rand crate needed for WASM).
/// Uses FNV-1a-style mixing on a u64 seed.
fn simple_hash(mut seed: u64) -> u64 {
    seed ^= seed >> 33;
    seed = seed.wrapping_mul(0xff51afd7ed558ccd);
    seed ^= seed >> 33;
    seed = seed.wrapping_mul(0xc4ceb9fe1a85ec53);
    seed ^= seed >> 33;
    seed
}

fn generate_shop_offers(ctx: &ReducerContext, count: usize) -> Vec<u64> {
    let active_units: Vec<u64> = ctx
        .db
        .unit()
        .iter()
        .filter(|u| u.status == ContentStatus::Active)
        .map(|u| u.id)
        .collect();

    if active_units.is_empty() {
        return vec![0; count];
    }

    // Seed from timestamp (nanos) — each reroll/floor gets a different timestamp
    let base_seed = ctx.timestamp.to_micros_since_unix_epoch() as u64;

    // Find current floor for extra entropy
    let floor_seed: u64 = ctx
        .db
        .game_match()
        .iter()
        .find(|m| m.player == ctx.sender())
        .map(|m| m.floor as u64)
        .unwrap_or(0);

    let mut offers = Vec::new();
    for i in 0..count {
        let seed = base_seed
            .wrapping_add(floor_seed.wrapping_mul(31))
            .wrapping_add(i as u64);
        let hash = simple_hash(seed);
        let idx = (hash as usize) % active_units.len();
        offers.push(active_units[idx]);
    }
    offers
}
