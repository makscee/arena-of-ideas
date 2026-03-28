use serde::{Deserialize, Serialize};

/// Unit tier (1-5). Determines stat budget and ability count.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Tier(u8);

/// Base stat budget per tier level (hp + pwr <= tier * BASE_BUDGET).
const BASE_BUDGET: i32 = 5;

impl Tier {
    pub fn new(value: u8) -> Option<Tier> {
        if (1..=5).contains(&value) {
            Some(Tier(value))
        } else {
            None
        }
    }

    pub fn value(self) -> u8 {
        self.0
    }

    /// Maximum combined hp + pwr for this tier.
    pub fn stat_budget(self) -> i32 {
        self.0 as i32 * BASE_BUDGET
    }

    /// Maximum number of abilities a unit of this tier can have.
    pub fn max_abilities(self) -> u8 {
        match self.0 {
            1..=2 => 1,
            3..=4 => 2,
            5 => 3,
            _ => unreachable!(),
        }
    }

    /// Whether the given stats fit within this tier's budget.
    pub fn stats_valid(self, hp: i32, pwr: i32) -> bool {
        hp > 0 && pwr > 0 && hp + pwr <= self.stat_budget()
    }
}

impl std::fmt::Display for Tier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Tier {}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tier_valid_range() {
        assert!(Tier::new(0).is_none());
        assert!(Tier::new(1).is_some());
        assert!(Tier::new(5).is_some());
        assert!(Tier::new(6).is_none());
    }

    #[test]
    fn tier_stat_budget() {
        assert_eq!(Tier::new(1).unwrap().stat_budget(), 5);
        assert_eq!(Tier::new(3).unwrap().stat_budget(), 15);
        assert_eq!(Tier::new(5).unwrap().stat_budget(), 25);
    }

    #[test]
    fn tier_max_abilities() {
        assert_eq!(Tier::new(1).unwrap().max_abilities(), 1);
        assert_eq!(Tier::new(2).unwrap().max_abilities(), 1);
        assert_eq!(Tier::new(3).unwrap().max_abilities(), 2);
        assert_eq!(Tier::new(4).unwrap().max_abilities(), 2);
        assert_eq!(Tier::new(5).unwrap().max_abilities(), 3);
    }

    #[test]
    fn tier_stats_valid() {
        let t1 = Tier::new(1).unwrap();
        assert!(t1.stats_valid(3, 2));  // 3 + 2 = 5 <= 5
        assert!(!t1.stats_valid(3, 3)); // 3 + 3 = 6 > 5
        assert!(!t1.stats_valid(0, 5)); // hp must be > 0
        assert!(!t1.stats_valid(5, 0)); // pwr must be > 0
    }

    #[test]
    fn tier_serde_roundtrip() {
        for i in 1..=5 {
            let tier = Tier::new(i).unwrap();
            let json = serde_json::to_string(&tier).unwrap();
            let deserialized: Tier = serde_json::from_str(&json).unwrap();
            assert_eq!(tier, deserialized);
        }
    }
}
