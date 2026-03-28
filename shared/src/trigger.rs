use serde::{Deserialize, Serialize};

/// When a unit's abilities fire during battle.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Trigger {
    /// At the start of battle, before any turns
    BattleStart,
    /// At the end of each turn
    TurnEnd,
    /// When this unit is about to die (hp <= 0)
    BeforeDeath,
    /// When an ally dies
    AllyDeath,
    /// Before this unit strikes
    BeforeStrike,
    /// After this unit strikes
    AfterStrike,
    /// When this unit takes damage
    DamageTaken,
    /// When this unit deals damage
    DamageDealt,
    /// Fires on multiple triggers
    Any(Vec<Trigger>),
}

impl std::fmt::Display for Trigger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Trigger::BattleStart => write!(f, "Battle Start"),
            Trigger::TurnEnd => write!(f, "Turn End"),
            Trigger::BeforeDeath => write!(f, "Before Death"),
            Trigger::AllyDeath => write!(f, "Ally Death"),
            Trigger::BeforeStrike => write!(f, "Before Strike"),
            Trigger::AfterStrike => write!(f, "After Strike"),
            Trigger::DamageTaken => write!(f, "Damage Taken"),
            Trigger::DamageDealt => write!(f, "Damage Dealt"),
            Trigger::Any(triggers) => {
                let names: Vec<String> = triggers.iter().map(|t| t.to_string()).collect();
                write!(f, "Any({})", names.join(", "))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trigger_serde_roundtrip() {
        let triggers = vec![
            Trigger::BattleStart,
            Trigger::TurnEnd,
            Trigger::BeforeDeath,
            Trigger::AllyDeath,
            Trigger::BeforeStrike,
            Trigger::AfterStrike,
            Trigger::DamageTaken,
            Trigger::DamageDealt,
            Trigger::Any(vec![Trigger::BeforeStrike, Trigger::DamageTaken]),
        ];

        for trigger in triggers {
            let json = serde_json::to_string(&trigger).unwrap();
            let deserialized: Trigger = serde_json::from_str(&json).unwrap();
            assert_eq!(trigger, deserialized);
        }
    }

    #[test]
    fn trigger_display() {
        assert_eq!(Trigger::BeforeStrike.to_string(), "Before Strike");
        assert_eq!(
            Trigger::Any(vec![Trigger::TurnEnd, Trigger::AllyDeath]).to_string(),
            "Any(Turn End, Ally Death)"
        );
    }
}
