use super::*;

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct UnitKillTrigger {
    pub damage_type: Option<DamageType>,
    #[serde(flatten)]
    pub effect: Effect,
}

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default, deny_unknown_fields)]
pub struct UnitTakeDamageTrigger {
    pub damage_type: Option<DamageType>,
    #[serde(flatten)]
    pub effect: Effect,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "on", deny_unknown_fields)]
pub enum UnitTrigger {
    Death(Effect),
    Spawn(Effect),
    Kill(UnitKillTrigger),
    TakeDamage(UnitTakeDamageTrigger),
}
