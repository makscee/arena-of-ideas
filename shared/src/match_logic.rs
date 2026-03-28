/// Match-related validation and logic that can be tested natively.

const STARTING_GOLD: i32 = 10;
const TEAM_SIZE: usize = 5;
const REROLL_COST: i32 = 1;

pub fn tier_cost(tier: u8) -> i32 {
    tier as i32
}

pub fn sell_value(tier: u8) -> i32 {
    (tier as i32).max(1)
}

pub fn floor_gold_reward(floor: u8) -> i32 {
    (floor as i32 + 2).min(10)
}

pub fn can_buy(gold: i32, tier: u8) -> bool {
    gold >= tier_cost(tier)
}

pub fn can_reroll(gold: i32) -> bool {
    gold >= REROLL_COST
}

pub fn team_is_full(team_size: usize) -> bool {
    team_size >= TEAM_SIZE
}

/// Check if stacking is valid (buying duplicate of existing team unit).
pub fn can_stack(team_unit_ids: &[u64], buying_unit_id: u64) -> bool {
    team_unit_ids.contains(&buying_unit_id)
}

/// Validate fusion parameters and return (result_tier, result_ability_count).
pub fn validate_fusion(
    copies_a: u8,
    is_fused_a: bool,
    is_fused_b: bool,
    tier_a: u8,
    tier_b: u8,
    abilities_a: &[u64],
    abilities_b: &[u64],
    chosen_abilities: &[u64],
) -> Result<(u8, usize), String> {
    if copies_a < 3 {
        return Err(format!("Unit A needs 3 copies to fuse, has {}", copies_a));
    }
    if is_fused_a || is_fused_b {
        return Err("Cannot fuse already-fused units".to_string());
    }

    let result_tier = tier_a.max(tier_b) + 1;
    if result_tier > 5 {
        return Err("Fusion would exceed max tier 5".to_string());
    }

    let result_ability_count = abilities_a.len().min(abilities_b.len()) + 1;

    if chosen_abilities.len() != result_ability_count {
        return Err(format!(
            "Must choose exactly {} abilities, got {}",
            result_ability_count,
            chosen_abilities.len()
        ));
    }

    let combined: Vec<u64> = abilities_a
        .iter()
        .chain(abilities_b.iter())
        .copied()
        .collect();
    for &chosen in chosen_abilities {
        if !combined.contains(&chosen) {
            return Err(format!("Ability {} not from either parent", chosen));
        }
    }

    Ok((result_tier, result_ability_count))
}

/// Validate feeding: donor abilities must be subset of fused abilities.
pub fn validate_feed(
    fused_abilities: &[u64],
    donor_abilities: &[u64],
    target_is_fused: bool,
) -> Result<(), String> {
    if !target_is_fused {
        return Err("Target must be a fused unit".to_string());
    }
    for &donor_ability in donor_abilities {
        if !fused_abilities.contains(&donor_ability) {
            return Err(format!("Donor ability {} not in fused unit", donor_ability));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== Economy =====

    #[test]
    fn tier_cost_scales_with_tier() {
        assert_eq!(tier_cost(1), 1);
        assert_eq!(tier_cost(3), 3);
        assert_eq!(tier_cost(5), 5);
    }

    #[test]
    fn sell_value_minimum_one() {
        assert_eq!(sell_value(0), 1);
        assert_eq!(sell_value(1), 1);
        assert_eq!(sell_value(3), 3);
    }

    #[test]
    fn floor_gold_caps_at_10() {
        assert_eq!(floor_gold_reward(1), 3);
        assert_eq!(floor_gold_reward(5), 7);
        assert_eq!(floor_gold_reward(10), 10);
        assert_eq!(floor_gold_reward(20), 10);
    }

    #[test]
    fn can_buy_checks_gold() {
        assert!(can_buy(5, 3));
        assert!(can_buy(3, 3));
        assert!(!can_buy(2, 3));
    }

    #[test]
    fn can_reroll_checks_gold() {
        assert!(can_reroll(1));
        assert!(!can_reroll(0));
    }

    #[test]
    fn team_full_at_five() {
        assert!(!team_is_full(4));
        assert!(team_is_full(5));
        assert!(team_is_full(6));
    }

    // ===== Stacking =====

    #[test]
    fn stack_detects_duplicate() {
        assert!(can_stack(&[1, 2, 3], 2));
        assert!(!can_stack(&[1, 2, 3], 4));
    }

    // ===== Fusion =====

    #[test]
    fn fusion_valid() {
        let result = validate_fusion(3, false, false, 2, 1, &[10], &[20], &[10, 20]);
        assert_eq!(result, Ok((3, 2)));
    }

    #[test]
    fn fusion_not_enough_copies() {
        assert!(validate_fusion(2, false, false, 1, 1, &[10], &[20], &[10, 20]).is_err());
    }

    #[test]
    fn fusion_already_fused_a() {
        assert!(validate_fusion(3, true, false, 1, 1, &[10], &[20], &[10, 20]).is_err());
    }

    #[test]
    fn fusion_already_fused_b() {
        assert!(validate_fusion(3, false, true, 1, 1, &[10], &[20], &[10, 20]).is_err());
    }

    #[test]
    fn fusion_exceeds_max_tier() {
        assert!(validate_fusion(3, false, false, 5, 4, &[10], &[20], &[10, 20]).is_err());
    }

    #[test]
    fn fusion_wrong_ability_count() {
        // min(1,1)+1 = 2, but chose 1
        assert!(validate_fusion(3, false, false, 2, 1, &[10], &[20], &[10]).is_err());
    }

    #[test]
    fn fusion_ability_not_from_parent() {
        assert!(validate_fusion(3, false, false, 2, 1, &[10], &[20], &[10, 99]).is_err());
    }

    #[test]
    fn fusion_multi_ability_parents() {
        // min(2,1)+1 = 2
        let result = validate_fusion(3, false, false, 3, 2, &[10, 20], &[30], &[10, 30]);
        assert_eq!(result, Ok((4, 2)));
    }

    #[test]
    fn fusion_tier_calculation() {
        // max(1, 3) + 1 = 4
        let result = validate_fusion(3, false, false, 1, 3, &[10], &[20], &[10, 20]);
        assert_eq!(result.unwrap().0, 4);
    }

    // ===== Feeding =====

    #[test]
    fn feed_valid_subset() {
        assert!(validate_feed(&[10, 20, 30], &[10, 20], true).is_ok());
    }

    #[test]
    fn feed_valid_single() {
        assert!(validate_feed(&[10, 20], &[10], true).is_ok());
    }

    #[test]
    fn feed_not_subset() {
        assert!(validate_feed(&[10, 20], &[10, 30], true).is_err());
    }

    #[test]
    fn feed_target_not_fused() {
        assert!(validate_feed(&[10, 20], &[10], false).is_err());
    }

    #[test]
    fn feed_empty_donor() {
        assert!(validate_feed(&[10, 20], &[], true).is_ok());
    }
}
