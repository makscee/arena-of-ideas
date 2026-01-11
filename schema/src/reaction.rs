use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Hash)]
#[serde(deny_unknown_fields)]
pub struct Behavior {
    pub trigger: Trigger,
    pub target: Target,
    pub effect: Effect,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Hash)]
#[serde(deny_unknown_fields)]
pub struct Effect {
    pub description: String,
    pub actions: Vec<Action>,
}

#[derive(
    Debug, Clone, Serialize, Deserialize, Default, PartialEq, Hash, EnumIter, AsRefStr, Display,
)]
pub enum Target {
    #[default]
    Owner,
    RandomEnemy,
    AllEnemies,
    RandomAlly,
    AllAllies,
    All,
    Caster,
    Attacker,
    Target,
    AdjacentBack,
    AdjacentFront,
    AllyAtSlot(u8),
    EnemyAtSlot(u8),
    List(Vec<Target>),
}

pub trait BehaviorTier {
    fn tier(&self) -> u8;
}

impl BehaviorTier for Behavior {
    fn tier(&self) -> u8 {
        let trigger_tier = self.trigger.tier();
        let target_tier = self.target.tier();
        let effect_tier = self.effect.actions.iter().map(|a| a.tier()).sum::<u8>();
        (trigger_tier + target_tier + effect_tier) / 3
    }
}

impl BehaviorTier for Target {
    fn tier(&self) -> u8 {
        match self {
            Target::Owner | Target::Caster => 0,
            Target::RandomEnemy | Target::RandomAlly | Target::Attacker | Target::Target => 1,
            Target::AdjacentBack
            | Target::AdjacentFront
            | Target::AllyAtSlot(_)
            | Target::EnemyAtSlot(_) => 2,
            Target::AllEnemies | Target::AllAllies => 3,
            Target::All => 4,
            Target::List(targets) => {
                let total: u8 = targets.iter().map(|t| t.tier()).sum();
                if targets.is_empty() {
                    0
                } else {
                    (total + targets.len() as u8 - 1) / targets.len() as u8
                }
            }
        }
    }
}
