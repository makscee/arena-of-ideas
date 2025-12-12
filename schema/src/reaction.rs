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
    AdjacentAlly,
    AllyAtSlot(u8),
    EnemyAtSlot(u8),
    List(Vec<Target>),
}

impl Target {
    pub fn to_expression(&self) -> Expression {
        match self {
            Target::Owner => Expression::owner,
            Target::RandomEnemy => Expression::random_unit(Expression::all_enemy_units.into()),
            Target::AllEnemies => Expression::all_enemy_units,
            Target::RandomAlly => Expression::random_unit(Expression::all_ally_units.into()),
            Target::AllAllies => Expression::all_ally_units,
            Target::All => Expression::all_units,
            Target::Caster => Expression::caster,
            Target::Attacker => Expression::attacker,
            Target::Target => Expression::target,
            Target::AdjacentAlly => Expression::adjacent_ally_units,
            Target::AllyAtSlot(slot) => Expression::value(VarValue::f32(*slot as f32)),
            Target::EnemyAtSlot(slot) => Expression::value(VarValue::f32(*slot as f32)),
            Target::List(targets) => {
                let expressions: Vec<Expression> =
                    targets.iter().map(|t| t.to_expression()).collect();
                Expression::list(expressions.into())
            }
        }
    }
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
            Target::AdjacentAlly | Target::AllyAtSlot(_) | Target::EnemyAtSlot(_) => 2,
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
