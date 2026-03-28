use serde::{Deserialize, Serialize};

use crate::content_status::ContentStatus;
use crate::tier::Tier;
use crate::trigger::Trigger;

/// A playable unit composed of a trigger, abilities, stats, and visuals.
/// Units are assembled by players from the active ability pool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Unit {
    pub id: u64,
    pub name: String,
    pub description: String,
    pub hp: i32,
    pub pwr: i32,
    pub tier: Tier,
    /// When this unit's abilities fire
    pub trigger: Trigger,
    /// Ability IDs this unit uses (1-3 based on tier)
    pub abilities: Vec<u64>,
    /// Rhai script for visual rendering
    pub painter_script: String,
    pub rating: i32,
    pub status: ContentStatus,
    /// Whether this unit was created via fusion (in-match only, not persisted)
    pub is_fused: bool,
}

impl Unit {
    /// Validate that this unit's stats and ability count fit its tier.
    pub fn validate(&self) -> Result<(), UnitValidationError> {
        if self.name.is_empty() {
            return Err(UnitValidationError::EmptyName);
        }
        if !self.tier.stats_valid(self.hp, self.pwr) {
            return Err(UnitValidationError::StatsOverBudget {
                hp: self.hp,
                pwr: self.pwr,
                budget: self.tier.stat_budget(),
            });
        }
        let max = self.tier.max_abilities() as usize;
        if self.abilities.is_empty() || self.abilities.len() > max {
            return Err(UnitValidationError::InvalidAbilityCount {
                count: self.abilities.len(),
                max,
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnitValidationError {
    EmptyName,
    StatsOverBudget { hp: i32, pwr: i32, budget: i32 },
    InvalidAbilityCount { count: usize, max: usize },
}

impl std::fmt::Display for UnitValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnitValidationError::EmptyName => write!(f, "Unit name cannot be empty"),
            UnitValidationError::StatsOverBudget { hp, pwr, budget } => {
                write!(f, "Stats {hp}hp + {pwr}pwr = {} exceeds budget {budget}", hp + pwr)
            }
            UnitValidationError::InvalidAbilityCount { count, max } => {
                write!(f, "Ability count {count} invalid (must be 1-{max})")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_unit(hp: i32, pwr: i32, tier: u8, abilities: Vec<u64>) -> Unit {
        Unit {
            id: 1,
            name: "Test Unit".to_string(),
            description: "A test".to_string(),
            hp,
            pwr,
            tier: Tier::new(tier).unwrap(),
            trigger: Trigger::BeforeStrike,
            abilities,
            painter_script: String::new(),
            rating: 0,
            status: ContentStatus::Draft,
            is_fused: false,
        }
    }

    #[test]
    fn valid_tier1_unit() {
        let unit = make_unit(3, 2, 1, vec![100]);
        assert!(unit.validate().is_ok());
    }

    #[test]
    fn tier1_over_budget() {
        let unit = make_unit(3, 3, 1, vec![100]);
        assert_eq!(
            unit.validate(),
            Err(UnitValidationError::StatsOverBudget { hp: 3, pwr: 3, budget: 5 })
        );
    }

    #[test]
    fn tier1_too_many_abilities() {
        let unit = make_unit(3, 2, 1, vec![100, 101]);
        assert_eq!(
            unit.validate(),
            Err(UnitValidationError::InvalidAbilityCount { count: 2, max: 1 })
        );
    }

    #[test]
    fn tier3_two_abilities_valid() {
        let unit = make_unit(8, 5, 3, vec![100, 101]);
        assert!(unit.validate().is_ok());
    }

    #[test]
    fn tier5_three_abilities_valid() {
        let unit = make_unit(15, 8, 5, vec![100, 101, 102]);
        assert!(unit.validate().is_ok());
    }

    #[test]
    fn empty_name_invalid() {
        let mut unit = make_unit(3, 2, 1, vec![100]);
        unit.name = String::new();
        assert_eq!(unit.validate(), Err(UnitValidationError::EmptyName));
    }

    #[test]
    fn no_abilities_invalid() {
        let unit = make_unit(3, 2, 1, vec![]);
        assert_eq!(
            unit.validate(),
            Err(UnitValidationError::InvalidAbilityCount { count: 0, max: 1 })
        );
    }

    #[test]
    fn unit_serde_roundtrip() {
        let unit = make_unit(5, 3, 2, vec![1, 2]);
        // tier 2 allows 1 ability, so this will fail validation
        // but serde should still work
        let json = serde_json::to_string(&unit).unwrap();
        let deserialized: Unit = serde_json::from_str(&json).unwrap();
        assert_eq!(unit.name, deserialized.name);
        assert_eq!(unit.hp, deserialized.hp);
        assert_eq!(unit.abilities.len(), deserialized.abilities.len());
    }
}
