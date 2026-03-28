/// Shared validation logic that mirrors server-side checks.
/// These can be tested natively (not WASM-only like server code).
use crate::tier::Tier;

/// Validate ability creation parameters.
pub fn validate_ability(
    name: &str,
    effect_script: &str,
    parent_a: u64,
    parent_b: u64,
) -> Result<(), String> {
    if name.is_empty() {
        return Err("Ability name cannot be empty".to_string());
    }
    if name.len() > 100 {
        return Err("Ability name too long (max 100)".to_string());
    }
    if effect_script.is_empty() {
        return Err("Effect script cannot be empty".to_string());
    }
    if effect_script.len() > 2000 {
        return Err("Effect script too long (max 2000 chars)".to_string());
    }
    if parent_a != 0 && parent_a == parent_b {
        return Err("Cannot breed an ability with itself".to_string());
    }
    Ok(())
}

/// Validate unit creation parameters.
pub fn validate_unit(
    name: &str,
    hp: i32,
    pwr: i32,
    tier: u8,
    ability_count: usize,
) -> Result<(), String> {
    if name.is_empty() {
        return Err("Unit name cannot be empty".to_string());
    }
    if name.len() > 100 {
        return Err("Unit name too long (max 100)".to_string());
    }

    let tier = Tier::new(tier).ok_or_else(|| format!("Invalid tier {} (must be 1-5)", tier))?;

    if hp <= 0 || pwr <= 0 {
        return Err("HP and PWR must be positive".to_string());
    }
    if !tier.stats_valid(hp, pwr) {
        return Err(format!(
            "Stats {}hp + {}pwr = {} exceeds tier {} budget of {}",
            hp,
            pwr,
            hp + pwr,
            tier,
            tier.stat_budget()
        ));
    }

    let max = tier.max_abilities() as usize;
    if ability_count == 0 || ability_count > max {
        return Err(format!(
            "{} units need 1-{} abilities, got {}",
            tier, max, ability_count
        ));
    }

    Ok(())
}

/// Validate vote parameters.
pub fn validate_vote(entity_kind: &str, value: i8) -> Result<(), String> {
    if entity_kind != "ability" && entity_kind != "unit" {
        return Err("entity_kind must be 'ability' or 'unit'".to_string());
    }
    if value != 1 && value != -1 {
        return Err("Vote value must be 1 or -1".to_string());
    }
    Ok(())
}

/// Validate fusion parameters.
pub fn validate_fusion(
    tier_a: u8,
    tier_b: u8,
    abilities_a: &[u64],
    abilities_b: &[u64],
    chosen_trigger_from_a: bool,
    chosen_abilities: &[u64],
) -> Result<(u8, u8), String> {
    // Result tier = max + 1
    let result_tier = tier_a.max(tier_b) + 1;
    if result_tier > 5 {
        return Err("Fusion would exceed max tier 5".to_string());
    }

    // Result ability count = min(a, b) + 1
    let result_ability_count = abilities_a.len().min(abilities_b.len()) + 1;

    if chosen_abilities.len() != result_ability_count {
        return Err(format!(
            "Must choose exactly {} abilities (min({}, {}) + 1), got {}",
            result_ability_count,
            abilities_a.len(),
            abilities_b.len(),
            chosen_abilities.len()
        ));
    }

    // All chosen abilities must come from parent pools
    let combined: Vec<u64> = abilities_a
        .iter()
        .chain(abilities_b.iter())
        .copied()
        .collect();
    for &chosen in chosen_abilities {
        if !combined.contains(&chosen) {
            return Err(format!(
                "Chosen ability {} not found in either parent",
                chosen
            ));
        }
    }

    let _ = chosen_trigger_from_a; // trigger choice is always valid if from either parent

    Ok((result_tier, result_ability_count as u8))
}

/// Validate feeding parameters.
pub fn validate_feed(fused_abilities: &[u64], donor_abilities: &[u64]) -> Result<(), String> {
    // All donor abilities must be a subset of fused unit's abilities
    for &donor_ability in donor_abilities {
        if !fused_abilities.contains(&donor_ability) {
            return Err(format!(
                "Donor ability {} is not in the fused unit's ability set",
                donor_ability
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== Ability Validation =====

    #[test]
    fn ability_valid() {
        assert!(validate_ability("Strike", "damage(X)", 0, 0).is_ok());
    }

    #[test]
    fn ability_empty_name() {
        assert_eq!(
            validate_ability("", "damage(X)", 0, 0),
            Err("Ability name cannot be empty".to_string())
        );
    }

    #[test]
    fn ability_name_too_long() {
        let long_name = "a".repeat(101);
        assert!(validate_ability(&long_name, "damage(X)", 0, 0).is_err());
    }

    #[test]
    fn ability_empty_script() {
        assert_eq!(
            validate_ability("Strike", "", 0, 0),
            Err("Effect script cannot be empty".to_string())
        );
    }

    #[test]
    fn ability_script_too_long() {
        let long_script = "x".repeat(2001);
        assert!(validate_ability("Strike", &long_script, 0, 0).is_err());
    }

    #[test]
    fn ability_same_parents() {
        assert_eq!(
            validate_ability("Child", "damage(X)", 5, 5),
            Err("Cannot breed an ability with itself".to_string())
        );
    }

    #[test]
    fn ability_different_parents_ok() {
        assert!(validate_ability("Child", "damage(X)", 1, 2).is_ok());
    }

    #[test]
    fn ability_no_parents_ok() {
        assert!(validate_ability("Primordial", "damage(X)", 0, 0).is_ok());
    }

    // ===== Unit Validation =====

    #[test]
    fn unit_valid_tier1() {
        assert!(validate_unit("Soldier", 3, 2, 1, 1).is_ok());
    }

    #[test]
    fn unit_valid_tier3_two_abilities() {
        assert!(validate_unit("Paladin", 8, 6, 3, 2).is_ok());
    }

    #[test]
    fn unit_valid_tier5_three_abilities() {
        assert!(validate_unit("Champion", 15, 8, 5, 3).is_ok());
    }

    #[test]
    fn unit_empty_name() {
        assert!(validate_unit("", 3, 2, 1, 1).is_err());
    }

    #[test]
    fn unit_invalid_tier_zero() {
        assert!(validate_unit("Bad", 3, 2, 0, 1).is_err());
    }

    #[test]
    fn unit_invalid_tier_six() {
        assert!(validate_unit("Bad", 3, 2, 6, 1).is_err());
    }

    #[test]
    fn unit_zero_hp() {
        assert!(validate_unit("Bad", 0, 3, 1, 1).is_err());
    }

    #[test]
    fn unit_zero_pwr() {
        assert!(validate_unit("Bad", 3, 0, 1, 1).is_err());
    }

    #[test]
    fn unit_over_budget() {
        // Tier 1 budget = 5, 3+3 = 6
        assert!(validate_unit("Bad", 3, 3, 1, 1).is_err());
    }

    #[test]
    fn unit_exact_budget() {
        // Tier 1 budget = 5, 3+2 = 5
        assert!(validate_unit("Good", 3, 2, 1, 1).is_ok());
    }

    #[test]
    fn unit_too_many_abilities_tier1() {
        assert!(validate_unit("Bad", 3, 2, 1, 2).is_err());
    }

    #[test]
    fn unit_too_many_abilities_tier3() {
        assert!(validate_unit("Bad", 8, 6, 3, 3).is_err());
    }

    #[test]
    fn unit_no_abilities() {
        assert!(validate_unit("Bad", 3, 2, 1, 0).is_err());
    }

    // ===== Vote Validation =====

    #[test]
    fn vote_valid_upvote_ability() {
        assert!(validate_vote("ability", 1).is_ok());
    }

    #[test]
    fn vote_valid_downvote_unit() {
        assert!(validate_vote("unit", -1).is_ok());
    }

    #[test]
    fn vote_invalid_kind() {
        assert!(validate_vote("house", 1).is_err());
    }

    #[test]
    fn vote_invalid_value() {
        assert!(validate_vote("ability", 0).is_err());
        assert!(validate_vote("ability", 2).is_err());
    }

    // ===== Fusion Validation =====

    #[test]
    fn fusion_valid() {
        // Tier 2 + Tier 1 → Tier 3, min(1,1)+1 = 2 abilities
        let result = validate_fusion(2, 1, &[10], &[20], true, &[10, 20]);
        assert_eq!(result, Ok((3, 2)));
    }

    #[test]
    fn fusion_would_exceed_max_tier() {
        assert!(validate_fusion(5, 4, &[10], &[20], true, &[10, 20]).is_err());
    }

    #[test]
    fn fusion_wrong_ability_count() {
        // min(1,1)+1 = 2, but only chose 1
        assert!(validate_fusion(2, 1, &[10], &[20], true, &[10]).is_err());
    }

    #[test]
    fn fusion_ability_not_from_parent() {
        assert!(validate_fusion(2, 1, &[10], &[20], true, &[10, 99]).is_err());
    }

    #[test]
    fn fusion_multi_ability_parents() {
        // Parent A: tier 3 with [10, 20], Parent B: tier 2 with [30]
        // min(2, 1) + 1 = 2 abilities
        let result = validate_fusion(3, 2, &[10, 20], &[30], true, &[10, 30]);
        assert_eq!(result, Ok((4, 2)));
    }

    // ===== Feed Validation =====

    #[test]
    fn feed_valid_subset() {
        // Fused has [10, 20, 30], donor has [10, 20] — subset
        assert!(validate_feed(&[10, 20, 30], &[10, 20]).is_ok());
    }

    #[test]
    fn feed_valid_single() {
        assert!(validate_feed(&[10, 20], &[10]).is_ok());
    }

    #[test]
    fn feed_invalid_not_subset() {
        // Fused has [10, 20], donor has [10, 30] — 30 not in fused
        assert!(validate_feed(&[10, 20], &[10, 30]).is_err());
    }

    #[test]
    fn feed_empty_donor_ok() {
        assert!(validate_feed(&[10, 20], &[]).is_ok());
    }
}
