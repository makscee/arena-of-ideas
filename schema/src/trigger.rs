use super::*;

#[derive(
    Debug, Clone, Serialize, Deserialize, Default, AsRefStr, PartialEq, Eq, Hash, Display, EnumIter,
)]
pub enum Trigger {
    #[default]
    BattleStart,
    TurnEnd,
    BeforeDeath,
    AllyDeath,
    BeforeStrike,
    AfterStrike,
    DamageTaken,
    DamageDealt,
    StatusApplied,
    StatusGained,
    ChangeStat(VarName),
    ChangeOutgoingDamage,
    ChangeIncomingDamage,
    Any(Vec<Trigger>),
}

impl Trigger {
    pub fn tier(&self) -> u8 {
        match self {
            Trigger::BattleStart
            | Trigger::TurnEnd
            | Trigger::BeforeDeath
            | Trigger::AllyDeath
            | Trigger::BeforeStrike
            | Trigger::AfterStrike
            | Trigger::DamageTaken
            | Trigger::DamageDealt
            | Trigger::StatusApplied
            | Trigger::StatusGained
            | Trigger::ChangeStat(_)
            | Trigger::ChangeOutgoingDamage
            | Trigger::ChangeIncomingDamage => 1,
            Trigger::Any(triggers) => {
                let total: u8 = triggers.iter().map(|t| t.tier()).sum();
                if triggers.is_empty() {
                    1
                } else {
                    (total + triggers.len() as u8 - 1) / triggers.len() as u8
                }
            }
        }
    }
}
