use serde::{Deserialize, Serialize};

use crate::content_status::ContentStatus;
use crate::target::TargetType;

/// A named, shared game mechanic with a Rhai effect script.
/// Abilities are the core creative content — bred by players via AI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ability {
    pub id: u64,
    pub name: String,
    pub description: String,
    pub target_type: TargetType,
    /// Rhai script that executes the ability's effect.
    /// Has access to: X (pwr), level (team ability count), owner, target, ctx
    pub effect_script: String,
    /// First parent ability (for evolution tree), None for primordial abilities
    pub parent_a: Option<u64>,
    /// Second parent ability (for evolution tree)
    pub parent_b: Option<u64>,
    pub rating: i32,
    pub status: ContentStatus,
    pub season: u32,
}

/// Calculate ability level from team unit count sharing this ability.
/// 1-2 units → level 1, 3-4 → level 2, 5 → level 3
pub fn ability_level(units_with_ability: u8) -> u8 {
    match units_with_ability {
        0 => 0,
        1..=2 => 1,
        3..=4 => 2,
        _ => 3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ability_serde_roundtrip() {
        let ability = Ability {
            id: 1,
            name: "Steal Gold".to_string(),
            description: "Deals damage and steals power".to_string(),
            target_type: TargetType::RandomEnemy,
            effect_script: "ability_actions.deal_damage(target, X * level);".to_string(),
            parent_a: None,
            parent_b: None,
            rating: 42,
            status: ContentStatus::Active,
            season: 1,
        };

        let json = serde_json::to_string(&ability).unwrap();
        let deserialized: Ability = serde_json::from_str(&json).unwrap();
        assert_eq!(ability.name, deserialized.name);
        assert_eq!(ability.id, deserialized.id);
        assert_eq!(ability.target_type, deserialized.target_type);
        assert_eq!(ability.status, deserialized.status);
    }

    #[test]
    fn ability_with_parents() {
        let child = Ability {
            id: 3,
            name: "Ember Heist".to_string(),
            description: "Fire theft".to_string(),
            target_type: TargetType::RandomEnemy,
            effect_script: String::new(),
            parent_a: Some(1),
            parent_b: Some(2),
            rating: 0,
            status: ContentStatus::Draft,
            season: 2,
        };

        assert_eq!(child.parent_a, Some(1));
        assert_eq!(child.parent_b, Some(2));
    }

    #[test]
    fn ability_level_thresholds() {
        assert_eq!(ability_level(0), 0);
        assert_eq!(ability_level(1), 1);
        assert_eq!(ability_level(2), 1);
        assert_eq!(ability_level(3), 2);
        assert_eq!(ability_level(4), 2);
        assert_eq!(ability_level(5), 3);
        assert_eq!(ability_level(10), 3); // caps at 3
    }
}
