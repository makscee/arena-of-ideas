use super::*;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct UnitKillTrigger {
    pub damage_type: Option<DamageType>,
    #[serde(flatten)]
    pub effect: Effect,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct UnitTakeDamageTrigger {
    pub damage_type: Option<DamageType>,
    #[serde(flatten)]
    pub effect: Effect,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct UnitShieldBrokenTrigger {
    pub heal: DamageValue,
}
