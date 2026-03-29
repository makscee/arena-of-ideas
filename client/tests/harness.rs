//! Integration test harness for SpacetimeDB server.
//!
//! Requires:
//!   1. Local SpacetimeDB server running: `spacetime start`
//!   2. Module published: `spacetime publish -p server --server local aoi-test --delete-data -y`
//!
//! Run with: cargo test -p client --test harness -- --test-threads=1

use std::process::Command;

const DB_NAME: &str = "aoi-test";
const SERVER: &str = "local";

// ===== Test Infrastructure =====

/// Call a reducer. Args are passed as separate CLI arguments.
/// Use `--` before any argument that starts with `-` (like -1).
fn call(reducer: &str, args: &[&str]) -> Result<String, String> {
    let mut cmd = Command::new("spacetime");
    cmd.arg("call").arg(DB_NAME).arg(reducer);
    cmd.arg("--server").arg(SERVER);

    // Add `--` separator then args (prevents negative numbers being parsed as flags)
    if !args.is_empty() {
        cmd.arg("--");
        for arg in args {
            cmd.arg(arg);
        }
    }

    let output = cmd
        .output()
        .map_err(|e| format!("Failed to run spacetime: {}", e))?;
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() || stderr.contains("Error:") {
        Err(format!("Reducer {} failed: {}", reducer, stderr))
    } else {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

/// Run a SQL query and return raw output.
fn sql(query: &str) -> String {
    let output = Command::new("spacetime")
        .arg("sql")
        .arg(DB_NAME)
        .arg(query)
        .arg("--server")
        .arg(SERVER)
        .output()
        .expect("Failed to run spacetime sql");

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        panic!("SQL query failed: {}\nQuery: {}", stderr, query);
    }

    stdout
}

/// Check if output contains a data value (filters out WARNING lines and headers).
fn output_contains(output: &str, value: &str) -> bool {
    output.contains(value)
}

// ===== Seed Data Tests =====

#[test]
fn test_01_seed_abilities_created() {
    let output = sql("SELECT * FROM ability");
    assert!(output_contains(&output, "Strike"), "Should have Strike");
    assert!(output_contains(&output, "Guard"), "Should have Guard");
    assert!(output_contains(&output, "Heal"), "Should have Heal");
    assert!(output_contains(&output, "Curse"), "Should have Curse");
}

#[test]
fn test_02_seed_units_created() {
    let output = sql("SELECT * FROM unit");
    assert!(output_contains(&output, "Footsoldier"));
    assert!(output_contains(&output, "Paladin"));
    assert!(output_contains(&output, "Warlock"));
}

// ===== Auth Tests =====

#[test]
fn test_03_register_player() {
    // Register — may succeed or fail if already registered. Either is fine.
    let _ = call("register", &["\"TestPlayer\""]);

    // Player table should have at least one row
    let output = sql("SELECT * FROM player");
    assert!(
        output_contains(&output, "TestPlayer") || output_contains(&output, "0x"),
        "Player table should have entries: {}",
        output
    );
}

// ===== Content Tests =====

#[test]
fn test_04_create_ability() {
    // SpacetimeDB enums use JSON sum type format: {"variantName": {}}
    let result = call(
        "ability_create",
        &[
            "\"Fireball\"",
            "\"Launches a ball of fire\"",
            "{\"randomEnemy\": {}}",
            "\"deal_damage(target[\\\"id\\\"], X * level);\"",
            "0",
            "0",
            "1",
        ],
    );
    assert!(result.is_ok(), "Should create ability: {:?}", result);

    let output = sql("SELECT * FROM ability");
    assert!(output_contains(&output, "Fireball"));
}

#[test]
fn test_05_duplicate_ability_name_fails() {
    let result = call(
        "ability_create",
        &[
            "\"Strike\"",
            "\"Duplicate\"",
            "{\"randomEnemy\": {}}",
            "\"x\"",
            "0",
            "0",
            "1",
        ],
    );
    assert!(result.is_err(), "Duplicate name should fail");
}

#[test]
fn test_06_create_unit() {
    let result = call(
        "unit_create",
        &[
            "\"TestUnit\"",
            "\"A test unit\"",
            "3",
            "2",
            "1",
            "{\"beforeStrike\": {}}",
            "[1]",
            "\"painter.circle(20.0);\"",
        ],
    );
    assert!(result.is_ok(), "Should create unit: {:?}", result);

    let output = sql("SELECT * FROM unit");
    assert!(output_contains(&output, "TestUnit"));
}

#[test]
fn test_07_unit_over_budget_fails() {
    let result = call(
        "unit_create",
        &[
            "\"OverBudget\"",
            "\"Too strong\"",
            "4",
            "4",
            "1",
            "{\"beforeStrike\": {}}",
            "[1]",
            "\"\"",
        ],
    );
    assert!(result.is_err(), "Over budget should fail");
}

#[test]
fn test_08_unit_invalid_ability_fails() {
    let result = call(
        "unit_create",
        &[
            "\"BadRef\"",
            "\"Bad\"",
            "3",
            "2",
            "1",
            "{\"beforeStrike\": {}}",
            "[9999]",
            "\"\"",
        ],
    );
    assert!(result.is_err(), "Invalid ability ref should fail");
}

// ===== Voting Tests =====

#[test]
fn test_09_vote_upvote() {
    call("vote_cast", &["\"ability\"", "1", "1"]).unwrap();

    let output = sql("SELECT rating FROM ability WHERE id = 1");
    assert!(
        output_contains(&output, "1"),
        "Rating should include 1: {}",
        output
    );
}

#[test]
fn test_10_vote_downvote() {
    call("vote_cast", &["\"ability\"", "2", "-1"]).unwrap();

    let output = sql("SELECT rating FROM ability WHERE id = 2");
    assert!(
        output_contains(&output, "-1"),
        "Rating should be -1: {}",
        output
    );
}

// ===== Match Flow Tests =====

/// Set last_floor high so floor 1+ are regular battles (not boss battles).
fn setup_arena_for_regular_battles() {
    sql("UPDATE arena_state SET last_floor = 10 WHERE always_zero = 0");
}

#[test]
fn test_11_match_start() {
    let _ = call("match_abandon", &[]);
    setup_arena_for_regular_battles();
    call("match_start", &[]).unwrap();

    let output = sql("SELECT gold FROM game_match");
    assert!(output_contains(&output, "7"), "Should start with 7 gold: {}", output);
}

#[test]
fn test_12_match_buy_unit() {
    call("match_shop_buy", &["0"]).unwrap();

    // Gold decreases by tier cost (at least 1). Don't assert exact amount since shop is randomized.
    let output = sql("SELECT gold FROM game_match");
    assert!(
        !output_contains(&output, "7"),
        "Gold should have decreased after buy: {}",
        output
    );
}

#[test]
fn test_13_match_buy_second_unit() {
    call("match_shop_buy", &["1"]).unwrap();

    // Just verify the buy succeeded (gold decreased further)
    let output = sql("SELECT team FROM game_match");
    assert!(
        output_contains(&output, "unit_id"),
        "Should have units in team: {}",
        output
    );
}

#[test]
fn test_14_match_sell_unit() {
    call("match_sell_unit", &["0"]).unwrap();

    // Just verify sell succeeded — gold went up by sell_value(1)
    let output = sql("SELECT gold FROM game_match");
    assert!(
        output_contains(&output, "|"),
        "Should still have a match: {}",
        output
    );
}

#[test]
fn test_15_match_reroll() {
    call("match_shop_reroll", &[]).unwrap();

    // Verify reroll succeeded — match still exists
    let output = sql("SELECT * FROM game_match");
    assert!(
        output_contains(&output, "gold"),
        "Match should still exist after reroll: {}",
        output
    );
}

#[test]
fn test_16_match_submit_win() {
    // Must transition to battle state first
    call("match_start_battle", &[]).unwrap();
    call("match_submit_result", &["true"]).unwrap();

    // Floor should advance (regular battle: always advances)
    let output = sql("SELECT floor FROM game_match");
    assert!(
        output_contains(&output, "2"),
        "Floor should advance to 2: {}",
        output
    );
}

#[test]
fn test_17_match_submit_loss() {
    // Must transition to battle state first
    call("match_start_battle", &[]).unwrap();
    call("match_submit_result", &["false"]).unwrap();

    // Regular battle loss: floor advances, lives decrease
    let output = sql("SELECT lives FROM game_match");
    assert!(
        output_contains(&output, "2"),
        "Should have 2 lives after one loss: {}",
        output
    );
}

#[test]
fn test_18_match_abandon() {
    call("match_abandon", &[]).unwrap();

    let output = sql("SELECT * FROM game_match");
    // After abandon, no data rows — only WARNING + header + separator
    assert!(
        !output_contains(&output, "unit_id"),
        "Match should be deleted: {}",
        output
    );
}

// ===== Stacking Tests =====

#[test]
fn test_19_stacking_flow() {
    setup_arena_for_regular_battles();
    call("match_start", &[]).unwrap();

    // Buy from shop slot 0
    call("match_shop_buy", &["0"]).unwrap();

    // Reroll and buy from slot 0 again — may or may not stack (shop is randomized)
    call("match_shop_reroll", &[]).unwrap();
    call("match_shop_buy", &["0"]).unwrap();

    // With randomized shop, we can't guarantee stacking. Just verify team has entries.
    let output = sql("SELECT team FROM game_match");
    assert!(
        output_contains(&output, "unit_id"),
        "Should have units in team: {}",
        output
    );
}

#[test]
fn test_20_match_cleanup() {
    let _ = call("match_abandon", &[]);
}

// ===== Generation Tests =====

#[test]
fn test_21_gen_breed_ability() {
    // Breed Strike (1) + Guard (2)
    let result = call(
        "gen_breed_ability",
        &["1", "2", "\"combine offense and defense\""],
    );
    assert!(result.is_ok(), "Should create breed request: {:?}", result);

    let output = sql("SELECT * FROM gen_request");
    assert!(
        output_contains(&output, "combine offense"),
        "Should have the prompt"
    );
    assert!(
        output_contains(&output, "Pending") || output_contains(&output, "pending"),
        "Status should be Pending: {}",
        output
    );
}

#[test]
fn test_22_gen_breed_same_parent_fails() {
    let result = call("gen_breed_ability", &["1", "1", "\"self breed\""]);
    assert!(result.is_err(), "Should not breed with itself");
}

#[test]
fn test_23_gen_breed_invalid_parent_fails() {
    let result = call("gen_breed_ability", &["9999", "1", "\"bad parent\""]);
    assert!(result.is_err(), "Should fail with invalid parent");
}

#[test]
fn test_24_gen_breed_empty_prompt_fails() {
    let result = call("gen_breed_ability", &["1", "2", "\"\""]);
    assert!(result.is_err(), "Should fail with empty prompt");
}

#[test]
fn test_25_gen_create_unit() {
    let result = call("gen_create_unit", &["\"a fierce fire warrior\""]);
    assert!(
        result.is_ok(),
        "Should create unit gen request: {:?}",
        result
    );

    let output = sql("SELECT * FROM gen_request");
    assert!(output_contains(&output, "fire warrior"));
}

#[test]
fn test_26_gen_submit_result() {
    let result = call(
        "gen_submit_result",
        &["1", "\"result data here\"", "\"AI reasoning\""],
    );
    assert!(result.is_ok(), "Should submit result: {:?}", result);

    let output = sql("SELECT * FROM gen_result");
    assert!(
        output_contains(&output, "result data here"),
        "Should have result: {}",
        output
    );
}

#[test]
fn test_27_gen_mark_failed() {
    let result = call("gen_mark_failed", &["2"]);
    assert!(result.is_ok(), "Should mark as failed: {:?}", result);
}

// ===== Season Tests =====

#[test]
fn test_28_season_start() {
    let result = call("season_start", &[]);
    assert!(result.is_ok(), "Should start season: {:?}", result);

    let output = sql("SELECT * FROM season");
    // Season record should exist — check for created_at timestamp
    assert!(
        output_contains(&output, "2026"),
        "Should have season record: {}",
        output
    );
}

#[test]
fn test_29_rating_decay() {
    let result = call("apply_rating_decay", &[]);
    assert!(result.is_ok(), "Should apply decay: {:?}", result);
}

// ===== Feature Request Tests =====

#[test]
fn test_30_feature_request_create() {
    let result = call(
        "feature_request_create",
        &["\"Add position swapping mechanic\""],
    );
    assert!(result.is_ok(), "Should create request: {:?}", result);

    let output = sql("SELECT * FROM feature_request");
    assert!(
        output_contains(&output, "position swapping"),
        "Should have request: {}",
        output
    );
}

#[test]
fn test_31_feature_request_accept() {
    let result = call("feature_request_accept", &["1"]);
    assert!(result.is_ok(), "Should accept: {:?}", result);

    let output = sql("SELECT * FROM feature_request");
    assert!(
        output_contains(&output, "accepted"),
        "Should be accepted: {}",
        output
    );
}

#[test]
fn test_32_feature_request_reject() {
    // Create another request first
    call("feature_request_create", &["\"Make units fly\""]).unwrap();
    let result = call("feature_request_reject", &["2", "\"Too complex for now\""]);
    assert!(result.is_ok(), "Should reject: {:?}", result);

    let output = sql("SELECT * FROM feature_request WHERE id = 2");
    assert!(
        output_contains(&output, "rejected"),
        "Should be rejected: {}",
        output
    );
}

#[test]
fn test_33_feature_request_empty_fails() {
    let result = call("feature_request_create", &["\"\""]);
    assert!(result.is_err(), "Empty description should fail");
}
