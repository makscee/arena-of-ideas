use serde::{Deserialize, Serialize};

/// Who an ability targets when it fires.
/// This is defined per-ability, not per-unit.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TargetType {
    /// A random living enemy
    RandomEnemy,
    /// All living enemies
    AllEnemies,
    /// A random living ally
    RandomAlly,
    /// All living allies
    AllAllies,
    /// The unit that owns this ability
    Owner,
    /// All living units (both sides)
    All,
    /// The unit that attacked (for reactive triggers)
    Attacker,
    /// The adjacent unit behind this one
    AdjacentBack,
    /// The adjacent unit in front of this one
    AdjacentFront,
    /// A specific ally slot
    AllyAtSlot(u8),
    /// A specific enemy slot
    EnemyAtSlot(u8),
}

impl std::fmt::Display for TargetType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TargetType::RandomEnemy => write!(f, "Random Enemy"),
            TargetType::AllEnemies => write!(f, "All Enemies"),
            TargetType::RandomAlly => write!(f, "Random Ally"),
            TargetType::AllAllies => write!(f, "All Allies"),
            TargetType::Owner => write!(f, "Self"),
            TargetType::All => write!(f, "All Units"),
            TargetType::Attacker => write!(f, "Attacker"),
            TargetType::AdjacentBack => write!(f, "Adjacent Back"),
            TargetType::AdjacentFront => write!(f, "Adjacent Front"),
            TargetType::AllyAtSlot(s) => write!(f, "Ally Slot {s}"),
            TargetType::EnemyAtSlot(s) => write!(f, "Enemy Slot {s}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_type_serde_roundtrip() {
        let targets = vec![
            TargetType::RandomEnemy,
            TargetType::AllEnemies,
            TargetType::RandomAlly,
            TargetType::AllAllies,
            TargetType::Owner,
            TargetType::All,
            TargetType::Attacker,
            TargetType::AdjacentBack,
            TargetType::AdjacentFront,
            TargetType::AllyAtSlot(2),
            TargetType::EnemyAtSlot(0),
        ];

        for target in targets {
            let json = serde_json::to_string(&target).unwrap();
            let deserialized: TargetType = serde_json::from_str(&json).unwrap();
            assert_eq!(target, deserialized);
        }
    }

    #[test]
    fn target_type_display() {
        assert_eq!(TargetType::RandomEnemy.to_string(), "Random Enemy");
        assert_eq!(TargetType::Owner.to_string(), "Self");
        assert_eq!(TargetType::AllyAtSlot(3).to_string(), "Ally Slot 3");
    }
}
