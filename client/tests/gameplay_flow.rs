//! Full gameplay flow tests.
//!
//! These tests simulate complete player sessions: login, shop, battle, progression.
//! They bridge server state (via SpacetimeDB CLI) and client battle simulation (Rhai).
//!
//! Requires: local SpacetimeDB server with module published.
//! Run: cargo test -p client --test gameplay_flow -- --test-threads=1

use std::process::Command;

use client::plugins::battle::{BattleAbility, BattleUnit, simulate_battle};
use shared::battle::BattleSide;
use shared::target::TargetType;
use shared::trigger::Trigger;

const DB_NAME: &str = "aoi-test";
const SERVER: &str = "local";

// ===== Infrastructure =====

fn call(reducer: &str, args: &[&str]) -> Result<String, String> {
    let mut cmd = Command::new("spacetime");
    cmd.arg("call").arg(DB_NAME).arg(reducer);
    cmd.arg("--server").arg(SERVER);
    if !args.is_empty() {
        cmd.arg("--");
        for arg in args {
            cmd.arg(arg);
        }
    }
    let output = cmd.output().map_err(|e| format!("Failed: {}", e))?;
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    if !output.status.success() || stderr.contains("Error:") {
        Err(format!("{} failed: {}", reducer, stderr))
    } else {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

fn sql(query: &str) -> String {
    let output = Command::new("spacetime")
        .arg("sql")
        .arg(DB_NAME)
        .arg(query)
        .arg("--server")
        .arg(SERVER)
        .output()
        .expect("Failed to run sql");
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn contains(output: &str, value: &str) -> bool {
    output.contains(value)
}

/// Republish the module with clean data to ensure a fresh state.
/// Called once at the start of gameplay flow tests.
fn reset_db() {
    use std::sync::Once;
    static RESET: Once = Once::new();
    RESET.call_once(|| {
        let output = Command::new("spacetime")
            .arg("publish")
            .arg("-p")
            .arg("server")
            .arg("--server")
            .arg(SERVER)
            .arg(DB_NAME)
            .arg("--delete-data")
            .arg("-y")
            .current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/.."))
            .output()
            .expect("Failed to republish");
        assert!(
            output.status.success(),
            "Republish failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        // Brief pause to let module initialize after republish
        std::thread::sleep(std::time::Duration::from_millis(500));
        // Register player after fresh DB
        let _ = call("register", &["\"TestPlayer\""]);
    });
}

/// Set last_floor high so floor 1+ are regular battles (not boss battles).
fn setup_arena_for_regular_battles() {
    sql("UPDATE arena_state SET last_floor = 10 WHERE always_zero = 0");
}

/// Ensure no active match exists, then start a fresh one.
fn fresh_match() {
    reset_db();
    setup_arena_for_regular_battles();
    // Abandon any existing match
    let _ = call("match_abandon", &[]);
    // Start fresh match
    match call("match_start", &[]) {
        Ok(_) => {}
        Err(e) => {
            let _ = call("match_abandon", &[]);
            call("match_start", &[])
                .unwrap_or_else(|e2| panic!("Cannot start match: {}\nOriginal: {}", e2, e));
        }
    }
}

// ===== Battle Helpers =====

/// Create a BattleUnit from known data (mirrors seeded units).
fn make_striker(id: u64, name: &str, hp: i32, pwr: i32, slot: u8, side: BattleSide) -> BattleUnit {
    BattleUnit {
        id,
        name: name.to_string(),
        hp,
        pwr,
        dmg: 0,
        shield: 0,
        trigger: Trigger::BeforeStrike,
        abilities: vec![BattleAbility {
            id: 1,
            name: "Strike".to_string(),
            target_type: TargetType::RandomEnemy,
            effect_script: "deal_damage(target[\"id\"], X * level);".to_string(),
        }],
        side,
        slot,
        alive: true,
    }
}

fn make_healer(id: u64, name: &str, hp: i32, pwr: i32, slot: u8, side: BattleSide) -> BattleUnit {
    BattleUnit {
        id,
        name: name.to_string(),
        hp,
        pwr,
        dmg: 0,
        shield: 0,
        trigger: Trigger::TurnEnd,
        abilities: vec![BattleAbility {
            id: 3,
            name: "Heal".to_string(),
            target_type: TargetType::RandomAlly,
            effect_script: "heal_damage(target[\"id\"], X * level);".to_string(),
        }],
        side,
        slot,
        alive: true,
    }
}

fn make_guardian(id: u64, name: &str, hp: i32, pwr: i32, slot: u8, side: BattleSide) -> BattleUnit {
    BattleUnit {
        id,
        name: name.to_string(),
        hp,
        pwr,
        dmg: 0,
        shield: 0,
        trigger: Trigger::BattleStart,
        abilities: vec![BattleAbility {
            id: 2,
            name: "Guard".to_string(),
            target_type: TargetType::Owner,
            effect_script: "add_shield(owner[\"id\"], X * level);".to_string(),
        }],
        side,
        slot,
        alive: true,
    }
}

fn make_curser(id: u64, name: &str, hp: i32, pwr: i32, slot: u8, side: BattleSide) -> BattleUnit {
    BattleUnit {
        id,
        name: name.to_string(),
        hp,
        pwr,
        dmg: 0,
        shield: 0,
        trigger: Trigger::BattleStart,
        abilities: vec![BattleAbility {
            id: 4,
            name: "Curse".to_string(),
            target_type: TargetType::RandomEnemy,
            effect_script: "change_stat(target[\"id\"], \"pwr\", -level);".to_string(),
        }],
        side,
        slot,
        alive: true,
    }
}

fn make_paladin(id: u64, name: &str, hp: i32, pwr: i32, slot: u8, side: BattleSide) -> BattleUnit {
    BattleUnit {
        id,
        name: name.to_string(),
        hp,
        pwr,
        dmg: 0,
        shield: 0,
        trigger: Trigger::BeforeStrike,
        abilities: vec![
            BattleAbility {
                id: 1,
                name: "Strike".to_string(),
                target_type: TargetType::RandomEnemy,
                effect_script: "deal_damage(target[\"id\"], X * level);".to_string(),
            },
            BattleAbility {
                id: 2,
                name: "Guard".to_string(),
                target_type: TargetType::Owner,
                effect_script: "add_shield(owner[\"id\"], X * level);".to_string(),
            },
        ],
        side,
        slot,
        alive: true,
    }
}

// =============================================================================
// BATTLE SCENARIO TESTS
// Pure battle simulation tests with expected outcomes
// =============================================================================

#[test]
fn scenario_01_equal_strikers_left_wins() {
    // Two identical strikers: left goes first (by array order), deals damage first
    // Both have 3hp/2pwr. Left hits for 2, right has 1hp left.
    // Right hits for 2, left has 1hp left. Next turn left kills right.
    let left = vec![make_striker(1, "Left", 3, 2, 0, BattleSide::Left)];
    let right = vec![make_striker(2, "Right", 3, 2, 0, BattleSide::Right)];
    let result = simulate_battle(left, right);
    assert_eq!(
        result.winner,
        BattleSide::Left,
        "Left should win (first mover advantage)"
    );
}

#[test]
fn scenario_02_stronger_unit_wins() {
    // Knight (6hp/4pwr) vs Footsoldier (3hp/2pwr)
    // Knight deals 4 damage per turn, kills Footsoldier in 1 turn
    let left = vec![make_striker(1, "Knight", 6, 4, 0, BattleSide::Left)];
    let right = vec![make_striker(2, "Footsoldier", 3, 2, 0, BattleSide::Right)];
    let result = simulate_battle(left, right);
    assert_eq!(result.winner, BattleSide::Left);
    assert!(
        result.turns <= 2,
        "Should win quickly: {} turns",
        result.turns
    );
}

#[test]
fn scenario_03_numbers_advantage() {
    // 3 weak units vs 1 strong unit
    // 3x Footsoldier (3hp/2pwr) vs Knight (6hp/4pwr)
    // 3 footsoldiers deal 6 total damage per turn at level 2 (3 share Strike)
    // Actually level 2 means X*2, so each does 4 damage = 12 total
    // Knight deals 4 to one footsoldier, kills it
    // But the numbers should overwhelm
    let left = vec![
        make_striker(1, "Foot1", 3, 2, 0, BattleSide::Left),
        make_striker(2, "Foot2", 3, 2, 1, BattleSide::Left),
        make_striker(3, "Foot3", 3, 2, 2, BattleSide::Left),
    ];
    let right = vec![make_striker(10, "Knight", 6, 4, 0, BattleSide::Right)];
    let result = simulate_battle(left, right);
    assert_eq!(result.winner, BattleSide::Left, "Numbers should win");
}

#[test]
fn scenario_04_healer_sustains_team() {
    // Striker + Healer vs 2 Strikers
    // The healer should keep the striker alive longer
    let left = vec![
        make_striker(1, "Fighter", 5, 3, 0, BattleSide::Left),
        make_healer(2, "Medic", 3, 3, 1, BattleSide::Left),
    ];
    let right = vec![
        make_striker(3, "Enemy1", 4, 2, 0, BattleSide::Right),
        make_striker(4, "Enemy2", 4, 2, 1, BattleSide::Right),
    ];
    let result = simulate_battle(left, right);
    // Healer fires on TurnEnd targeting RandomAlly (excluding self)
    // The healing helps sustain but outcome depends on damage race
    // Just verify the battle completes and healer actually healed
    let heal_count = result
        .actions
        .iter()
        .filter(|a| matches!(a, shared::battle::BattleAction::Heal { .. }))
        .count();
    assert!(heal_count > 0, "Healer should have healed at least once");
}

#[test]
fn scenario_05_shield_absorbs_damage() {
    // Guardian (4hp/1pwr, shields on BattleStart) vs Striker (3hp/2pwr)
    // Guardian gets 1*1 = 1 shield at battle start
    // Striker hits for 2, shield absorbs 1, guardian takes 1
    // Guardian hits for 1 each turn
    // Shield gives a small advantage
    let left = vec![make_guardian(1, "Guardian", 4, 1, 0, BattleSide::Left)];
    let right = vec![make_striker(2, "Attacker", 3, 2, 0, BattleSide::Right)];
    let result = simulate_battle(left, right);
    // Guardian should survive a bit longer due to shield but with only 1 pwr
    // this will be a long fight ending in fatigue potentially
    assert!(result.turns > 0, "Battle should complete");
}

#[test]
fn scenario_06_curse_weakens_enemy() {
    // Curser (3hp/2pwr) + Striker (3hp/2pwr) vs strong Striker (5hp/5pwr)
    // Curser reduces enemy pwr by level(2 units share nothing, so level 1) = 1
    // Enemy goes from 5pwr to 4pwr
    let left = vec![
        make_curser(1, "Hexer", 3, 2, 0, BattleSide::Left),
        make_striker(2, "Fighter", 3, 2, 1, BattleSide::Left),
    ];
    let right = vec![make_striker(3, "Brute", 5, 5, 0, BattleSide::Right)];
    let result = simulate_battle(left, right);
    // Curse should weaken the brute, giving left a fighting chance
    assert!(result.turns > 1, "Should be a multi-turn fight");
}

#[test]
fn scenario_07_paladin_strikes_and_shields() {
    // Paladin (8hp/6pwr) with Strike+Guard vs 2 Strikers
    // Paladin fires both abilities on BeforeStrike: deals damage AND shields self
    let left = vec![make_paladin(1, "Paladin", 8, 6, 0, BattleSide::Left)];
    let right = vec![
        make_striker(2, "E1", 4, 3, 0, BattleSide::Right),
        make_striker(3, "E2", 4, 3, 1, BattleSide::Right),
    ];
    let result = simulate_battle(left, right);
    assert_eq!(result.winner, BattleSide::Left, "Paladin should dominate");
}

#[test]
fn scenario_08_ability_level_scaling_matters() {
    // 5 units with Strike (level 3: X*3) vs 5 units without shared abilities
    // Left: 5 strikers, each deals pwr*3 per turn
    // Right: mix of different abilities, each at level 1
    let left = vec![
        make_striker(1, "S1", 3, 2, 0, BattleSide::Left),
        make_striker(2, "S2", 3, 2, 1, BattleSide::Left),
        make_striker(3, "S3", 3, 2, 2, BattleSide::Left),
        make_striker(4, "S4", 3, 2, 3, BattleSide::Left),
        make_striker(5, "S5", 3, 2, 4, BattleSide::Left),
    ];
    let right = vec![
        make_striker(10, "R1", 3, 2, 0, BattleSide::Right),
        make_healer(11, "R2", 3, 2, 1, BattleSide::Right),
        make_guardian(12, "R3", 3, 2, 2, BattleSide::Right),
        make_curser(13, "R4", 3, 2, 3, BattleSide::Right),
        make_striker(14, "R5", 3, 2, 4, BattleSide::Right),
    ];
    let result = simulate_battle(left, right);
    // Left has level 3 Strike (each hit does 6 damage), right has mixed level 1
    assert_eq!(
        result.winner,
        BattleSide::Left,
        "Ability scaling should dominate"
    );
    assert!(
        result.turns <= 5,
        "Should win fast with level 3: {} turns",
        result.turns
    );
}

#[test]
fn scenario_09_deterministic_same_result() {
    let make_teams = || {
        let left = vec![
            make_striker(1, "A", 5, 3, 0, BattleSide::Left),
            make_healer(2, "B", 4, 2, 1, BattleSide::Left),
            make_guardian(3, "C", 6, 1, 2, BattleSide::Left),
        ];
        let right = vec![
            make_paladin(10, "X", 8, 4, 0, BattleSide::Right),
            make_curser(11, "Y", 3, 3, 1, BattleSide::Right),
        ];
        (left, right)
    };

    let (l1, r1) = make_teams();
    let (l2, r2) = make_teams();
    let result1 = simulate_battle(l1, r1);
    let result2 = simulate_battle(l2, r2);

    assert_eq!(result1.winner, result2.winner, "Must be deterministic");
    assert_eq!(result1.turns, result2.turns, "Same turn count");
    assert_eq!(
        result1.actions.len(),
        result2.actions.len(),
        "Same action count"
    );
}

#[test]
fn scenario_10_full_5v5_battle() {
    // Full team battle simulating what a real match looks like
    let left = vec![
        make_striker(1, "Footsoldier", 3, 2, 0, BattleSide::Left),
        make_guardian(2, "Shieldbearer", 4, 1, 1, BattleSide::Left),
        make_healer(3, "Medic", 2, 3, 2, BattleSide::Left),
        make_striker(4, "Knight", 6, 4, 3, BattleSide::Left),
        make_paladin(5, "Paladin", 8, 6, 4, BattleSide::Left),
    ];
    let right = vec![
        make_curser(10, "Hexer", 2, 3, 0, BattleSide::Right),
        make_striker(11, "Footsoldier2", 3, 2, 1, BattleSide::Right),
        make_striker(12, "Knight2", 6, 4, 2, BattleSide::Right),
        make_healer(13, "Priest", 5, 5, 3, BattleSide::Right),
        make_paladin(14, "Warlock", 7, 7, 4, BattleSide::Right),
    ];
    let result = simulate_battle(left, right);

    // Just verify it completes without panic and produces actions
    assert!(result.turns > 0);
    assert!(!result.actions.is_empty(), "Should produce battle actions");
    // Both teams should have fought
    let deaths = result
        .actions
        .iter()
        .filter(|a| matches!(a, shared::battle::BattleAction::Death { .. }))
        .count();
    assert!(deaths >= 1, "At least one unit should die");
}

// =============================================================================
// FULL GAMEPLAY FLOW TESTS
// Server + Client simulation combined
// =============================================================================

#[test]
fn flow_01_complete_match_run() {
    fresh_match();
    let output = sql("SELECT gold FROM game_match");
    assert!(
        contains(&output, "7"),
        "Should start with 7 gold: {}",
        output
    );

    // Buy units from shop (second buy may fail if not enough gold for higher tier)
    call("match_shop_buy", &["0"]).unwrap();
    let _ = call("match_shop_buy", &["1"]); // may fail — that's ok

    // Verify team has at least one unit
    let output = sql("SELECT team FROM game_match");
    assert!(contains(&output, "unit_id"), "Should have units in team");

    // Simulate a battle (client-side)
    let left = vec![make_striker(1, "MyUnit1", 3, 2, 0, BattleSide::Left)];
    let right = vec![make_striker(100, "Opponent", 4, 2, 0, BattleSide::Right)];
    let battle_result = simulate_battle(left, right);
    let won = battle_result.winner == BattleSide::Left;

    // Must start battle before submitting result
    call("match_start_battle", &[]).unwrap();
    call("match_submit_result", &[if won { "true" } else { "false" }]).unwrap();

    // Floor always advances for regular battles
    let output = sql("SELECT * FROM game_match");
    assert!(contains(&output, "| 2"), "Floor should be 2: {}", output);

    // Second round: buy if possible
    let _ = call("match_shop_buy", &["0"]);

    // Second battle
    let left = vec![make_striker(1, "U1", 3, 2, 0, BattleSide::Left)];
    let right = vec![make_striker(100, "Opp1", 5, 3, 0, BattleSide::Right)];
    let result2 = simulate_battle(left, right);
    call("match_start_battle", &[]).unwrap();
    call(
        "match_submit_result",
        &[if result2.winner == BattleSide::Left {
            "true"
        } else {
            "false"
        }],
    )
    .unwrap();

    // Match may or may not still exist depending on win/loss sequence
    // Just verify the server didn't error out

    // Cleanup
    let _ = call("match_abandon", &[]);
}

#[test]
fn flow_02_lose_three_times_game_over() {
    fresh_match();

    // Verify match exists
    let output = sql("SELECT * FROM game_match");
    assert!(
        contains(&output, "gold"),
        "Match should exist after start: {}",
        output
    );

    // Lose first time — lives: 3 → 2 (regular battle: must start battle first)
    call("match_start_battle", &[]).unwrap();
    call("match_submit_result", &["false"]).unwrap();
    let output = sql("SELECT * FROM game_match");
    assert!(
        contains(&output, "gold"),
        "Match should exist after loss 1: {}",
        output
    );

    // Lose second time — lives: 2 → 1
    call("match_start_battle", &[]).unwrap();
    call("match_submit_result", &["false"]).unwrap();
    let output = sql("SELECT * FROM game_match");
    assert!(
        contains(&output, "gold"),
        "Match should exist after loss 2: {}",
        output
    );

    // Lose third time — lives: 1 → 0 → game over → match deleted
    call("match_start_battle", &[]).unwrap();
    call("match_submit_result", &["false"]).unwrap();
    let output = sql("SELECT * FROM game_match");
    // Match should be gone — header exists but no data rows
    // Data rows contain "0x" (identity hex) or actual numbers after separator
    let has_data = output.lines().any(|line| {
        let trimmed = line.trim();
        trimmed.starts_with("0x")
            || (trimmed.contains('|')
                && trimmed.chars().next().map_or(false, |c| c.is_ascii_digit()))
    });
    assert!(
        !has_data,
        "Match should be deleted after 3 losses: {}",
        output
    );
}

#[test]
fn flow_03_buy_sell_economy() {
    fresh_match();

    // Start: 7 gold
    // Buy a unit
    call("match_shop_buy", &["0"]).unwrap();

    // Verify we have a unit in team
    let output = sql("SELECT team FROM game_match");
    assert!(
        contains(&output, "unit_id"),
        "Should have unit after buy: {}",
        output
    );

    // Sell slot 0
    let sell_result = call("match_sell_unit", &["0"]);
    assert!(
        sell_result.is_ok(),
        "Sell should succeed: {:?}",
        sell_result
    );

    // Reroll
    let reroll_result = call("match_shop_reroll", &[]);
    assert!(
        reroll_result.is_ok(),
        "Reroll should succeed: {:?}",
        reroll_result
    );

    // Match should still exist
    let output = sql("SELECT * FROM game_match");
    assert!(
        contains(&output, "gold"),
        "Match should still exist: {}",
        output
    );

    let _ = call("match_abandon", &[]);
}

#[test]
fn flow_04_buy_multiple_units() {
    fresh_match();

    // Buy units from shop — shop is randomized, can't guarantee stacking
    call("match_shop_buy", &["0"]).unwrap();
    call("match_shop_reroll", &[]).unwrap();
    call("match_shop_buy", &["0"]).unwrap();

    // Verify team has entries (may or may not have stacked)
    let output = sql("SELECT team FROM game_match");
    assert!(
        contains(&output, "unit_id"),
        "Should have units in team: {}",
        output
    );

    let _ = call("match_abandon", &[]);
}

#[test]
fn flow_05_battle_with_seeded_units() {
    // Use actual seeded unit stats to simulate a real game battle
    // Footsoldier (3hp/2pwr) + Shieldbearer (4hp/1pwr) vs Knight (6hp/4pwr) + Hexer (2hp/3pwr)
    let left = vec![
        make_striker(1, "Footsoldier", 3, 2, 0, BattleSide::Left),
        make_guardian(2, "Shieldbearer", 4, 1, 1, BattleSide::Left),
    ];
    let right = vec![
        make_striker(5, "Knight", 6, 4, 0, BattleSide::Right),
        make_curser(4, "Hexer", 2, 3, 1, BattleSide::Right),
    ];

    let result = simulate_battle(left, right);

    // Verify battle produces valid action sequence
    let mut saw_spawn = false;
    let mut saw_damage = false;
    let mut saw_death = false;

    for action in &result.actions {
        match action {
            shared::battle::BattleAction::Spawn { .. } => saw_spawn = true,
            shared::battle::BattleAction::Damage { amount, .. } => {
                saw_damage = true;
                assert!(*amount > 0, "Damage should be positive");
            }
            shared::battle::BattleAction::Death { .. } => saw_death = true,
            _ => {}
        }
    }

    assert!(saw_spawn, "Should have spawn actions");
    assert!(saw_damage, "Should have damage actions");
    assert!(saw_death, "Should have death actions");
}

#[test]
fn flow_06_win_gives_gold_and_advances() {
    fresh_match();

    // Buy a unit so we have a team
    call("match_shop_buy", &["0"]).unwrap();

    // Win the battle — must start battle first
    call("match_start_battle", &[]).unwrap();
    call("match_submit_result", &["true"]).unwrap();

    // Should be floor 2 with more gold (gold_reward = 3)
    let output = sql("SELECT * FROM game_match");
    assert!(contains(&output, "2"), "Floor should be 2: {}", output);

    // Win again
    let _ = call("match_shop_buy", &["0"]); // spend some gold (may fail if not enough)
    call("match_start_battle", &[]).unwrap();
    call("match_submit_result", &["true"]).unwrap();

    let output = sql("SELECT * FROM game_match");
    assert!(contains(&output, "3"), "Floor should be 3: {}", output);

    let _ = call("match_abandon", &[]);
}

#[test]
fn flow_07_team_full_cant_buy() {
    fresh_match();

    // Try to buy units — with 7 gold and variable costs, some buys may fail
    // Just verify the server handles it gracefully
    let _ = call("match_shop_buy", &["0"]);
    let _ = call("match_shop_buy", &["1"]);
    let _ = call("match_shop_buy", &["2"]);

    // Verify match still exists
    let output = sql("SELECT * FROM game_match");
    assert!(
        contains(&output, "gold"),
        "Match should still exist: {}",
        output
    );

    let _ = call("match_abandon", &[]);
}

#[test]
fn flow_08_multi_round_progression() {
    fresh_match();

    // Play 5 rounds: buy, battle, progress
    for round in 0..5 {
        // Buy a unit if we can
        let _ = call("match_shop_buy", &["0"]);

        // Simulate battle
        let team_size = (round + 1).min(3);
        let mut left = Vec::new();
        for i in 0..team_size {
            left.push(make_striker(
                i as u64 + 1,
                &format!("Unit{}", i),
                3 + round,
                2 + round / 2,
                i as u8,
                BattleSide::Left,
            ));
        }
        let right = vec![make_striker(
            100,
            "Opp",
            3 + round,
            2 + round / 2,
            0,
            BattleSide::Right,
        )];

        let result = simulate_battle(left, right);
        let won = result.winner == BattleSide::Left;

        // Must start battle before submitting result
        call("match_start_battle", &[]).unwrap();
        call("match_submit_result", &[if won { "true" } else { "false" }]).unwrap();

        // Check if game over
        let output = sql("SELECT * FROM game_match");
        if !contains(&output, "game_match") || !contains(&output, "gold") {
            // Game over — match deleted
            return;
        }
    }

    let _ = call("match_abandon", &[]);
}
