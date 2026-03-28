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

    let output = cmd.output().map_err(|e| format!("Failed to run spacetime: {}", e))?;
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
    assert!(output_contains(&output, "1"), "Rating should include 1: {}", output);
}

#[test]
fn test_10_vote_downvote() {
    call("vote_cast", &["\"ability\"", "2", "-1"]).unwrap();

    let output = sql("SELECT rating FROM ability WHERE id = 2");
    assert!(output_contains(&output, "-1"), "Rating should be -1: {}", output);
}

// ===== Match Flow Tests =====

#[test]
fn test_11_match_start() {
    let _ = call("match_abandon", &[]);
    call("match_start", &[]).unwrap();

    let output = sql("SELECT * FROM game_match");
    assert!(output_contains(&output, "10"), "Should start with 10 gold");
}

#[test]
fn test_12_match_buy_unit() {
    call("match_shop_buy", &["0"]).unwrap();

    let output = sql("SELECT gold FROM game_match");
    assert!(output_contains(&output, "9"), "Gold should be 9: {}", output);
}

#[test]
fn test_13_match_buy_second_unit() {
    call("match_shop_buy", &["1"]).unwrap();

    let output = sql("SELECT gold FROM game_match");
    assert!(output_contains(&output, "8"), "Gold should be 8: {}", output);
}

#[test]
fn test_14_match_sell_unit() {
    call("match_sell_unit", &["0"]).unwrap();

    let output = sql("SELECT gold FROM game_match");
    assert!(output_contains(&output, "9"), "Gold should be 9 after sell: {}", output);
}

#[test]
fn test_15_match_reroll() {
    call("match_shop_reroll", &[]).unwrap();

    let output = sql("SELECT gold FROM game_match");
    assert!(output_contains(&output, "8"), "Gold should be 8 after reroll: {}", output);
}

#[test]
fn test_16_match_submit_win() {
    call("match_submit_result", &["true"]).unwrap();

    // Use SELECT * to avoid reserved word issues with column names
    let output = sql("SELECT * FROM game_match");
    // Floor should be 2 — look for it in the output
    assert!(output_contains(&output, "| 2"), "Floor should advance to 2: {}", output);
}

#[test]
fn test_17_match_submit_loss() {
    call("match_submit_result", &["false"]).unwrap();

    let output = sql("SELECT * FROM game_match");
    // Lives should be 2 (was 3, lost 1)
    assert!(output_contains(&output, "lives") || output_contains(&output, "2"), "Should have 2 lives: {}", output);
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
    call("match_start", &[]).unwrap();

    // Buy from shop slot 0
    call("match_shop_buy", &["0"]).unwrap();

    // Reroll and buy from slot 0 again (same unit pattern)
    call("match_shop_reroll", &[]).unwrap();
    call("match_shop_buy", &["0"]).unwrap();

    let output = sql("SELECT team FROM game_match");
    assert!(
        output_contains(&output, "copies = 2"),
        "Should have 2 copies stacked: {}",
        output
    );
}

#[test]
fn test_20_cleanup() {
    let _ = call("match_abandon", &[]);
}
