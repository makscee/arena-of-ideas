use super::*;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Default, Clone, AsRefStr)]
pub enum Event {
    #[default]
    BattleStart,
    TurnStart,
    TurnEnd,
    BeforeStrike(Entity, Entity),
    AfterStrike(Entity, Entity),
    Death(Entity),
    Kill {
        owner: Entity,
        target: Entity,
    },
    IncomingDamage {
        owner: Entity,
        value: i32,
    },
    DamageTaken {
        owner: Entity,
        value: i32,
    },
    OutgoingDamage {
        owner: Entity,
        target: Entity,
        value: i32,
    },
    DamageDealt {
        owner: Entity,
        target: Entity,
        value: i32,
    },
    Summon(Entity),
    UseAbility(String),
    ApplyStatus(String),
}
