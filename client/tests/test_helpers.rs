/// Structured test helpers for SpacetimeDB integration tests.
/// Wraps CLI calls and parses SQL output into usable data.
use std::process::Command;

pub const DB_NAME: &str = "aoi-test";
pub const SERVER: &str = "local";

/// Call a reducer, return Ok(stdout) or Err(stderr).
pub fn call(reducer: &str, args: &[&str]) -> Result<String, String> {
    let mut cmd = Command::new("spacetime");
    cmd.arg("call")
        .arg(DB_NAME)
        .arg(reducer)
        .arg("--server")
        .arg(SERVER);
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

/// Run SQL query, return raw output.
pub fn sql(query: &str) -> String {
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

/// Check if output contains a value.
pub fn contains(output: &str, value: &str) -> bool {
    output.contains(value)
}

/// Check if a table has any data rows.
pub fn has_data(output: &str) -> bool {
    output.lines().any(|line| {
        let t = line.trim();
        t.contains("0x")
            || (t.contains('|') && t.chars().next().map_or(false, |c| c.is_ascii_digit()))
    })
}

/// Count data rows in SQL output (excludes header, separator, warnings).
pub fn count_data_rows(output: &str) -> usize {
    output
        .lines()
        .filter(|line| {
            let t = line.trim();
            !t.is_empty()
                && !t.starts_with("WARNING")
                && !t.starts_with('-')
                && !t.contains("---")
                && (t.contains("0x") || t.chars().next().map_or(false, |c| c.is_ascii_digit()))
        })
        .count()
}

/// Parse a single-column numeric value from SQL output.
pub fn parse_int_value(output: &str) -> Option<i64> {
    for line in output.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with("WARNING") || t.starts_with('-') || t.contains("---") {
            continue;
        }
        // Try to parse as integer
        if let Ok(v) = t.parse::<i64>() {
            return Some(v);
        }
        // Try extracting number after pipe
        if let Some(val) = t.split('|').nth(0) {
            if let Ok(v) = val.trim().parse::<i64>() {
                return Some(v);
            }
        }
    }
    None
}

/// Republish the module with clean data.
pub fn republish() {
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
}

/// Set arena last_floor high so floor 1+ are regular battles.
pub fn setup_regular_battles() {
    sql("UPDATE arena_state SET last_floor = 10 WHERE always_zero = 0");
}

/// Ensure clean match state: register, abandon old, start new.
pub fn start_fresh_match() {
    let _ = call("register", &["\"TestPlayer\""]);
    let _ = call("match_abandon", &[]);
    setup_regular_battles();
    match call("match_start", &[]) {
        Ok(_) => {}
        Err(_) => {
            let _ = call("match_abandon", &[]);
            call("match_start", &[]).expect("Cannot start match");
        }
    }
}

/// Check if an active match exists for the current player.
pub fn match_exists() -> bool {
    let output = sql("SELECT * FROM game_match");
    has_data(&output)
}

// ===== Structured Assertions =====

/// Assert a table contains a specific value.
pub fn assert_table_contains(table: &str, value: &str) {
    let output = sql(&format!("SELECT * FROM {}", table));
    assert!(
        contains(&output, value),
        "{} should contain '{}': {}",
        table,
        value,
        output
    );
}

/// Assert a table has at least N data rows.
pub fn assert_table_has_rows(table: &str, min_rows: usize) {
    let output = sql(&format!("SELECT * FROM {}", table));
    let rows = count_data_rows(&output);
    assert!(
        rows >= min_rows,
        "{} should have at least {} rows, got {}: {}",
        table,
        min_rows,
        rows,
        output
    );
}

/// Assert a reducer call succeeds.
pub fn assert_call_ok(reducer: &str, args: &[&str]) {
    let result = call(reducer, args);
    assert!(result.is_ok(), "{} should succeed: {:?}", reducer, result);
}

/// Assert a reducer call fails.
pub fn assert_call_err(reducer: &str, args: &[&str]) {
    let result = call(reducer, args);
    assert!(result.is_err(), "{} should fail but succeeded", reducer);
}

/// Assert match has a specific gold amount.
pub fn assert_gold(expected: i32) {
    let output = sql("SELECT gold FROM game_match");
    assert!(
        contains(&output, &expected.to_string()),
        "Gold should be {}: {}",
        expected,
        output
    );
}

/// Assert match is on a specific floor.
pub fn assert_floor(expected: u8) {
    let output = sql("SELECT * FROM game_match");
    assert!(
        contains(&output, &format!("| {}", expected)),
        "Floor should be {}: {}",
        expected,
        output
    );
}
