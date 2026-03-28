use spacetimedb::{ReducerContext, Table};

#[allow(unused_imports)]
use crate::{ContentStatus, GameMatch, TeamSlot, Unit, ability, game_match, player, unit};

// ===== Match Table =====
// Match table is defined in lib.rs. This module contains the reducers.

const STARTING_GOLD: i32 = 10;
const STARTING_LIVES: i32 = 3;
const SHOP_SIZE: usize = 3;
const TEAM_SIZE: usize = 5;
const REROLL_COST: i32 = 1;

/// Start a new match run for the calling player.
#[spacetimedb::reducer]
pub fn match_start(ctx: &ReducerContext) -> Result<(), String> {
    // Check player exists
    if ctx.db.player().identity().find(ctx.sender()).is_none() {
        return Err("Player not registered".to_string());
    }

    // Check no active match
    for m in ctx.db.game_match().iter() {
        if m.player == ctx.sender() {
            return Err("Player already has an active match".to_string());
        }
    }

    // Generate shop offers from active units
    let shop_offers = generate_shop_offers(ctx, SHOP_SIZE);

    ctx.db.game_match().insert(crate::GameMatch {
        id: 0,
        player: ctx.sender(),
        floor: 1,
        gold: STARTING_GOLD,
        lives: STARTING_LIVES,
        team: Vec::new(),
        shop_offers,
        created_at: ctx.timestamp,
    });

    log::info!("Match started for {:?}", ctx.sender());
    Ok(())
}

/// Buy a unit from the shop at the given index.
#[spacetimedb::reducer]
pub fn match_shop_buy(ctx: &ReducerContext, shop_index: u32) -> Result<(), String> {
    let mut game_match = find_player_match(ctx)?;

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
        .ok_or_else(|| "Unit not found".to_string())?;

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
        game_match.team.push(crate::TeamSlot {
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
    game_match.shop_offers[idx] = 0; // Mark slot as bought

    ctx.db.game_match().id().update(game_match);
    Ok(())
}

/// Sell a unit from the team at the given slot index.
#[spacetimedb::reducer]
pub fn match_sell_unit(ctx: &ReducerContext, slot_index: u32) -> Result<(), String> {
    let mut game_match = find_player_match(ctx)?;

    let idx = slot_index as usize;
    if idx >= game_match.team.len() {
        return Err("Invalid slot index".to_string());
    }

    let slot = &game_match.team[idx];
    let tier = if slot.is_fused {
        slot.fused_tier
    } else {
        ctx.db
            .unit()
            .id()
            .find(slot.unit_id)
            .map(|u| u.tier)
            .unwrap_or(1)
    };

    game_match.gold += sell_value(tier);
    game_match.team.remove(idx);

    ctx.db.game_match().id().update(game_match);
    Ok(())
}

/// Reroll the shop for new offers.
#[spacetimedb::reducer]
pub fn match_shop_reroll(ctx: &ReducerContext) -> Result<(), String> {
    let mut game_match = find_player_match(ctx)?;

    if game_match.gold < REROLL_COST {
        return Err("Not enough gold to reroll".to_string());
    }

    game_match.gold -= REROLL_COST;
    game_match.shop_offers = generate_shop_offers(ctx, SHOP_SIZE);

    ctx.db.game_match().id().update(game_match);
    Ok(())
}

/// Move a unit from one slot to another (swap).
#[spacetimedb::reducer]
pub fn match_move_unit(ctx: &ReducerContext, from_slot: u32, to_slot: u32) -> Result<(), String> {
    let mut game_match = find_player_match(ctx)?;

    let from = from_slot as usize;
    let to = to_slot as usize;

    if from >= game_match.team.len() || to >= game_match.team.len() {
        return Err("Invalid slot index".to_string());
    }

    game_match.team.swap(from, to);
    ctx.db.game_match().id().update(game_match);
    Ok(())
}

/// Submit a battle result (called by client after simulation).
#[spacetimedb::reducer]
pub fn match_submit_result(ctx: &ReducerContext, won: bool) -> Result<(), String> {
    let mut game_match = find_player_match(ctx)?;

    if won {
        game_match.floor += 1;
        game_match.gold += floor_gold_reward(game_match.floor);
        // Refresh shop
        game_match.shop_offers = generate_shop_offers(ctx, SHOP_SIZE);
    } else {
        game_match.lives -= 1;
        if game_match.lives <= 0 {
            // Game over — delete match
            ctx.db.game_match().id().delete(game_match.id);
            log::info!("Match ended (defeat) for {:?}", ctx.sender());
            return Ok(());
        }
        // Refresh shop even on loss
        game_match.shop_offers = generate_shop_offers(ctx, SHOP_SIZE);
    }

    ctx.db.game_match().id().update(game_match);
    Ok(())
}

/// Abandon the current match.
#[spacetimedb::reducer]
pub fn match_abandon(ctx: &ReducerContext) -> Result<(), String> {
    let game_match = find_player_match(ctx)?;
    ctx.db.game_match().id().delete(game_match.id);
    log::info!("Match abandoned by {:?}", ctx.sender());
    Ok(())
}

/// Fuse two units on the team.
/// slot_a must have 3+ copies (fuseable).
/// Player chooses trigger_from_a (bool) and which abilities to keep.
#[spacetimedb::reducer]
pub fn match_fuse_units(
    ctx: &ReducerContext,
    slot_a: u32,
    slot_b: u32,
    trigger_from_a: bool,
    chosen_abilities: Vec<u64>,
) -> Result<(), String> {
    let mut game_match = find_player_match(ctx)?;

    let a = slot_a as usize;
    let b = slot_b as usize;

    if a >= game_match.team.len() || b >= game_match.team.len() || a == b {
        return Err("Invalid slot indices".to_string());
    }

    let slot_a_data = &game_match.team[a];
    let slot_b_data = &game_match.team[b];

    // Slot A must be fuseable (3+ copies)
    if slot_a_data.copies < 3 {
        return Err("Unit A needs 3 copies to fuse".to_string());
    }
    if slot_a_data.is_fused || slot_b_data.is_fused {
        return Err("Cannot fuse already-fused units".to_string());
    }

    let unit_a = ctx
        .db
        .unit()
        .id()
        .find(slot_a_data.unit_id)
        .ok_or("Unit A not found")?;
    let unit_b = ctx
        .db
        .unit()
        .id()
        .find(slot_b_data.unit_id)
        .ok_or("Unit B not found")?;

    // Calculate result
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

    // Validate chosen abilities come from parents
    let combined: Vec<u64> = abilities_a
        .iter()
        .chain(abilities_b.iter())
        .copied()
        .collect();
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

    // Remove slot B first (higher index if b > a)
    let (remove_first, remove_second) = if a > b { (a, b) } else { (b, a) };
    game_match.team.remove(remove_first);
    let fuse_idx = if a > b {
        remove_second
    } else {
        a.min(game_match.team.len())
    };

    // Update slot A to be fused
    if fuse_idx < game_match.team.len() {
        let slot = &mut game_match.team[fuse_idx];
        slot.is_fused = true;
        slot.fused_trigger = trigger_str;
        slot.fused_abilities = chosen_abilities;
        slot.fused_tier = result_tier;
        slot.copies = 1;
        // Bonus stats from fusion
        slot.bonus_hp += unit_b.hp / 2;
        slot.bonus_pwr += unit_b.pwr / 2;
    }

    ctx.db.game_match().id().update(game_match);
    Ok(())
}

/// Feed a donor unit into a fused unit for stat boost.
#[spacetimedb::reducer]
pub fn match_feed_unit(
    ctx: &ReducerContext,
    fused_slot: u32,
    donor_slot: u32,
) -> Result<(), String> {
    let mut game_match = find_player_match(ctx)?;

    let f = fused_slot as usize;
    let d = donor_slot as usize;

    if f >= game_match.team.len() || d >= game_match.team.len() || f == d {
        return Err("Invalid slot indices".to_string());
    }

    if !game_match.team[f].is_fused {
        return Err("Target must be a fused unit".to_string());
    }

    // Check donor abilities are subset of fused abilities
    let donor_unit = ctx
        .db
        .unit()
        .id()
        .find(game_match.team[d].unit_id)
        .ok_or("Donor unit not found")?;

    for &donor_ability in &donor_unit.abilities {
        if !game_match.team[f].fused_abilities.contains(&donor_ability) {
            return Err(format!("Donor ability {} not in fused unit", donor_ability));
        }
    }

    // Apply stat boost
    game_match.team[f].bonus_hp += 2;
    game_match.team[f].bonus_pwr += 1;

    // Remove donor
    game_match.team.remove(d);

    ctx.db.game_match().id().update(game_match);
    Ok(())
}

// ===== Helpers =====

fn find_player_match(ctx: &ReducerContext) -> Result<crate::GameMatch, String> {
    for m in ctx.db.game_match().iter() {
        if m.player == ctx.sender() {
            return Ok(m);
        }
    }
    Err("No active match found".to_string())
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

    // Simple deterministic selection: distribute across available units
    let mut offers = Vec::new();
    for i in 0..count {
        let idx = i % active_units.len();
        offers.push(active_units[idx]);
    }
    offers
}

fn tier_cost(tier: u8) -> i32 {
    tier as i32
}

fn sell_value(tier: u8) -> i32 {
    (tier as i32).max(1)
}

fn floor_gold_reward(floor: u8) -> i32 {
    (floor as i32 + 2).min(10)
}
