/// Battle helper functions shared between client and server.
/// The full simulation (Rhai execution) stays in client,
/// but data structures and analysis utilities live here.
use crate::battle::{BattleAction, BattleResult, BattleSide};

/// Count total damage dealt in a battle.
pub fn total_damage(result: &BattleResult) -> i32 {
    result
        .actions
        .iter()
        .filter_map(|a| match a {
            BattleAction::Damage { amount, .. } => Some(*amount),
            _ => None,
        })
        .sum()
}

/// Count deaths in a battle.
pub fn death_count(result: &BattleResult) -> usize {
    result
        .actions
        .iter()
        .filter(|a| matches!(a, BattleAction::Death { .. }))
        .count()
}

/// Count how many times a specific ability was used.
pub fn ability_use_count(result: &BattleResult, ability_name: &str) -> usize {
    result
        .actions
        .iter()
        .filter(|a| {
            matches!(a, BattleAction::AbilityUsed { ability_name: name, .. } if name == ability_name)
        })
        .count()
}

/// Check if a specific unit died.
pub fn unit_died(result: &BattleResult, unit_id: u64) -> bool {
    result
        .actions
        .iter()
        .any(|a| matches!(a, BattleAction::Death { unit } if *unit == unit_id))
}

/// Get total healing done.
pub fn total_healing(result: &BattleResult) -> i32 {
    result
        .actions
        .iter()
        .filter_map(|a| match a {
            BattleAction::Heal { amount, .. } => Some(*amount),
            _ => None,
        })
        .sum()
}

/// Verify battle result is valid (has spawns, ends with winner).
pub fn validate_result(result: &BattleResult) -> Result<(), String> {
    let spawn_count = result
        .actions
        .iter()
        .filter(|a| matches!(a, BattleAction::Spawn { .. }))
        .count();

    if spawn_count == 0 && result.turns > 0 {
        return Err("Battle has turns but no spawns".to_string());
    }

    if result.turns == 0 && !result.actions.is_empty() {
        return Err("Battle has actions but 0 turns".to_string());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_result() -> BattleResult {
        BattleResult {
            winner: BattleSide::Left,
            turns: 3,
            actions: vec![
                BattleAction::Spawn {
                    unit: 1,
                    slot: 0,
                    side: BattleSide::Left,
                },
                BattleAction::Spawn {
                    unit: 2,
                    slot: 0,
                    side: BattleSide::Right,
                },
                BattleAction::AbilityUsed {
                    source: 1,
                    ability_name: "Strike".to_string(),
                },
                BattleAction::Damage {
                    source: 1,
                    target: 2,
                    amount: 5,
                },
                BattleAction::AbilityUsed {
                    source: 2,
                    ability_name: "Strike".to_string(),
                },
                BattleAction::Damage {
                    source: 2,
                    target: 1,
                    amount: 3,
                },
                BattleAction::AbilityUsed {
                    source: 1,
                    ability_name: "Strike".to_string(),
                },
                BattleAction::Damage {
                    source: 1,
                    target: 2,
                    amount: 5,
                },
                BattleAction::Death { unit: 2 },
            ],
        }
    }

    #[test]
    fn test_total_damage() {
        assert_eq!(total_damage(&sample_result()), 13);
    }

    #[test]
    fn test_death_count() {
        assert_eq!(death_count(&sample_result()), 1);
    }

    #[test]
    fn test_ability_use_count() {
        assert_eq!(ability_use_count(&sample_result(), "Strike"), 3);
        assert_eq!(ability_use_count(&sample_result(), "Heal"), 0);
    }

    #[test]
    fn test_unit_died() {
        assert!(unit_died(&sample_result(), 2));
        assert!(!unit_died(&sample_result(), 1));
    }

    #[test]
    fn test_total_healing() {
        assert_eq!(total_healing(&sample_result()), 0);
    }

    #[test]
    fn test_validate_result() {
        assert!(validate_result(&sample_result()).is_ok());

        let empty = BattleResult {
            winner: BattleSide::Left,
            turns: 0,
            actions: vec![],
        };
        assert!(validate_result(&empty).is_ok());
    }
}
