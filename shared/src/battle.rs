use serde::{Deserialize, Serialize};

/// A single action that occurs during a battle.
/// The battle simulation produces a sequence of these.
/// The client animates them in order.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BattleAction {
    /// Unit deals damage to another unit
    Damage { source: u64, target: u64, amount: i32 },
    /// Unit heals another unit
    Heal { source: u64, target: u64, amount: i32 },
    /// Unit dies
    Death { unit: u64 },
    /// Unit spawns / appears at start of battle
    Spawn { unit: u64, slot: u8, side: BattleSide },
    /// A stat changes on a unit
    StatChange { unit: u64, stat: StatKind, delta: i32 },
    /// Visual effect
    Vfx { unit: u64, effect: String },
    /// Pause for animation timing
    Wait { seconds: f32 },
    /// Fatigue damage applied to both teams
    Fatigue { amount: i32 },
    /// An ability is used (for UI display)
    AbilityUsed { source: u64, ability_name: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BattleSide {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StatKind {
    Hp,
    Pwr,
    Dmg,
}

/// The result of a completed battle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleResult {
    pub winner: BattleSide,
    pub actions: Vec<BattleAction>,
    pub turns: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn battle_action_serde_roundtrip() {
        let actions = vec![
            BattleAction::Spawn { unit: 1, slot: 0, side: BattleSide::Left },
            BattleAction::Damage { source: 1, target: 2, amount: 5 },
            BattleAction::Heal { source: 1, target: 1, amount: 3 },
            BattleAction::Death { unit: 2 },
            BattleAction::StatChange { unit: 1, stat: StatKind::Pwr, delta: 2 },
            BattleAction::Vfx { unit: 1, effect: "fire".to_string() },
            BattleAction::Wait { seconds: 0.5 },
            BattleAction::Fatigue { amount: 1 },
            BattleAction::AbilityUsed { source: 1, ability_name: "Steal Gold".to_string() },
        ];

        for action in actions {
            let json = serde_json::to_string(&action).unwrap();
            let _deserialized: BattleAction = serde_json::from_str(&json).unwrap();
        }
    }

    #[test]
    fn battle_result_serde_roundtrip() {
        let result = BattleResult {
            winner: BattleSide::Left,
            actions: vec![
                BattleAction::Spawn { unit: 1, slot: 0, side: BattleSide::Left },
                BattleAction::Damage { source: 1, target: 2, amount: 3 },
                BattleAction::Death { unit: 2 },
            ],
            turns: 5,
        };

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: BattleResult = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.winner, BattleSide::Left);
        assert_eq!(deserialized.turns, 5);
        assert_eq!(deserialized.actions.len(), 3);
    }
}
