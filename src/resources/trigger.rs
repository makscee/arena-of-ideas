use super::*;

#[derive(Deserialize, Serialize, Clone, Debug, Display, PartialEq, EnumIter)]
#[serde(deny_unknown_fields)]
pub enum Trigger {
    Fire {
        #[serde(default)]
        triggers: Vec<(FireTrigger, Option<String>)>,
        #[serde(default)]
        targets: Vec<(Expression, Option<String>)>,
        #[serde(default)]
        effects: Vec<(Effect, Option<String>)>,
    },
    Change {
        trigger: DeltaTrigger,
        expr: Expression,
    },
    List(Vec<Box<Trigger>>),
}

#[derive(Deserialize, Serialize, Clone, Debug, Display, PartialEq, EnumIter, Default)]
pub enum DeltaTrigger {
    #[default]
    IncomingDamage,
    Var(VarName),
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, EnumIter, Default, AsRefStr)]
pub enum FireTrigger {
    #[default]
    Noop,
    List(Vec<Box<FireTrigger>>),
    Period(usize, usize, Box<FireTrigger>),
    OnceAfter(i32, Box<FireTrigger>),
    UnitUsedAbility(String),
    AllyUsedAbility(String),
    EnemyUsedAbility(String),
    AfterIncomingDamage,
    AfterDamageTaken,
    AfterDamageDealt,
    BattleStart,
    TurnStart,
    TurnEnd,
    BeforeStrike,
    AfterStrike,
    AllyDeath,
    AnyDeath,
    AllySummon,
    EnemySummon,
    BeforeDeath,
    AfterKill,
}
