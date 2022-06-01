use super::*;

mod attack_speed;
mod aura;
mod charmed;
mod detect;
mod freeze;
mod gained_effect;
mod invulnerability;
mod modifier;
mod on_deal_damage;
mod on_death;
mod on_heal;
mod on_kill;
mod on_shield_broken;
mod on_spawn;
mod on_take_damage;
mod protection;
mod repeating_effect;
mod scavenge;
mod self_detect;
mod shield;
mod slow;
mod stun;
mod taunt;
mod vulnerability;

pub use attack_speed::*;
pub use aura::*;
pub use charmed::*;
pub use detect::*;
pub use freeze::*;
pub use gained_effect::*;
pub use invulnerability::*;
pub use modifier::*;
pub use on_deal_damage::*;
pub use on_death::*;
pub use on_heal::*;
pub use on_kill::*;
pub use on_shield_broken::*;
pub use on_spawn::*;
pub use on_take_damage::*;
pub use protection::*;
pub use repeating_effect::*;
pub use scavenge::*;
pub use self_detect::*;
pub use shield::*;
pub use slow::*;
pub use stun::*;
pub use taunt::*;
pub use vulnerability::*;

// #[derive(Debug, Serialize, Deserialize, Copy, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
// pub enum StatusType {
//     Freeze,
//     Stun,
//     Shield,
//     Vulnerability,
//     Invulnerability,
//     Slow,
//     Modifier,
//     Aura,
//     Protection,
//     Detect,
//     SelfDetect,
//     Taunt,
//     OnDeath,
//     OnSpawn,
//     OnKill,
//     OnHeal,
//     OnDealDamage,
//     OnTakeDamage,
//     OnShieldBroken,
//     GainedEffect,
//     Scavenge,
//     AttackSpeed,
//     RepeatingEffect,
//     Charmed,
//     Bleed,
//     Plague,
//     SiphonLife,
// }

// #[derive(Debug, Serialize, Deserialize, Clone)]
// #[serde(tag = "type")]
// pub enum StatusOld {
//     Freeze(Box<FreezeStatus>),
//     Stun(Box<StunStatus>),
//     Shield(Box<ShieldStatus>),
//     Vulnerability(Box<VulnerabilityStatus>),
//     Invulnerability(Box<InvulnerabilityStatus>),
//     Slow(Box<SlowStatus>),
//     Modifier(Box<ModifierStatus>),
//     Aura(Box<AuraStatus>),
//     Protection(Box<ProtectionStatus>),
//     Detect(Box<DetectStatus>),
//     SelfDetect(Box<SelfDetectStatus>),
//     Taunt(Box<TauntStatus>),
//     OnDeath(Box<OnDeathStatus>),
//     OnSpawn(Box<OnSpawnStatus>),
//     OnKill(Box<OnKillStatus>),
//     OnHeal(Box<OnHealStatus>),
//     OnDealDamage(Box<OnDealDamageStatus>),
//     OnTakeDamage(Box<OnTakeDamageStatus>),
//     OnShieldBroken(Box<OnShieldBrokenStatus>),
//     GainedEffect(Box<GainedEffectStatus>),
//     Scavenge(Box<ScavengeStatus>),
//     AttackSpeed(Box<AttackSpeedStatus>),
//     RepeatingEffect(Box<RepeatingEffectStatus>),
//     Charmed(Box<CharmedStatus>),
//     Bleed(Box<RepeatingEffectStatus>),
//     Plague(Box<RepeatingEffectStatus>),
//     SiphonLife(Box<RepeatingEffectStatus>),
// }

/// Describes what to do when several equal statuses are attached to the same unit
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum StatusStacking {
    /// Statuses are attached independently and treated as different
    Independent,
    /// New status only refreshes the timer
    Refresh,
    /// New status only increases the stack counter variable
    Count,
    /// New status refreshes the timer and increases the stack counter variable
    CountRefresh,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum StatusTrigger {
    /// Triggered when the owner deals damage of the specified type (or any type if none is specified)
    DamageDealt { damage_type: Option<DamageType> },
    /// Triggered when the owner takes damage of the specified type (or any type if none is specified)
    DamageTaken { damage_type: Option<DamageType> },
    /// Triggered when the owner's shield breaks
    ShieldBroken,
    /// Triggered when the unit spawns
    Spawn,
    /// Triggered when the owner dies
    Death,
    /// Triggered when the owner is healed
    Heal,
    /// Triggered when the owner kills another unit
    Kill,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct StatusListener {
    /// A list of triggers for the effect
    pub triggers: Vec<StatusTrigger>,
    /// The effect to apply to the owner when triggered
    pub effect: Effect,
}

pub type StatusName = String;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Status {
    /// The name is used when comparing two statuses for equality for a stack
    /// and for parsing in the unit config
    pub name: StatusName,
    pub stacking: StatusStacking,
    /// If specified, the status will drop after that time,
    /// otherwise the status will be attached indefinitely
    /// or until it gets removed manually
    pub duration: Option<Time>,
    /// A list of triggers for this status
    pub triggers: Vec<StatusTrigger>,
    /// Specifications of effects to apply for different subsets of triggers
    pub listeners: Vec<StatusListener>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct AttachedStatus {
    /// The actual status that hold all the neccessary logic info
    pub status: Status,
    /// Specifies how much time is left until the status is dropped.
    /// If `None`, then the status remains attached.
    pub time: Option<Time>,
    /// Specifies the owner of the status
    pub owner: Option<Id>,
    /// Specifies the caster that applied the status
    pub caster: Option<Id>,
    /// Variables that persist for the lifetime of the status
    pub vars: HashMap<VarName, R32>,
}

impl Status {
    /// Transforms config into an attached status
    pub fn attach(self, owner: Option<Id>, caster: Option<Id>) -> AttachedStatus {
        AttachedStatus {
            time: self.duration,
            status: self,
            owner,
            caster,
            vars: default(),
        }
    }
}

impl EffectContainer for Status {
    fn walk_effects_mut(&mut self, f: &mut dyn FnMut(&mut Effect)) {
        self.listeners
            .iter_mut()
            .map(|listener| &mut listener.effect)
            .for_each(|effect| f(effect))
    }
}

pub trait StatusImpl: EffectContainer {}

// impl StatusOld {
//     pub fn as_mut(&mut self) -> &mut dyn StatusImpl {
//         match self {
//             Self::Freeze(status) => &mut **status,
//             Self::Stun(status) => &mut **status,
//             Self::Shield(status) => &mut **status,
//             Self::Vulnerability(status) => &mut **status,
//             Self::Invulnerability(status) => &mut **status,
//             Self::Slow(status) => &mut **status,
//             Self::Modifier(status) => &mut **status,
//             Self::Aura(status) => &mut **status,
//             Self::Protection(status) => &mut **status,
//             Self::Detect(status) => &mut **status,
//             Self::SelfDetect(status) => &mut **status,
//             Self::Taunt(status) => &mut **status,
//             Self::OnDeath(status) => &mut **status,
//             Self::OnSpawn(status) => &mut **status,
//             Self::OnKill(status) => &mut **status,
//             Self::OnHeal(status) => &mut **status,
//             Self::OnDealDamage(status) => &mut **status,
//             Self::OnTakeDamage(status) => &mut **status,
//             Self::OnShieldBroken(status) => &mut **status,
//             Self::GainedEffect(status) => &mut **status,
//             Self::Scavenge(status) => &mut **status,
//             Self::AttackSpeed(status) => &mut **status,
//             Self::RepeatingEffect(status) => &mut **status,
//             Self::Charmed(status) => &mut **status,
//             Self::Bleed(status) => &mut **status,
//             Self::Plague(status) => &mut **status,
//             Self::SiphonLife(status) => &mut **status,
//         }
//     }
//     pub fn as_box(self) -> Box<dyn StatusImpl> {
//         match self {
//             Self::Freeze(status) => status,
//             Self::Stun(status) => status,
//             Self::Shield(status) => status,
//             Self::Vulnerability(status) => status,
//             Self::Invulnerability(status) => status,
//             Self::Slow(status) => status,
//             Self::Modifier(status) => status,
//             Self::Aura(status) => status,
//             Self::Protection(status) => status,
//             Self::Detect(status) => status,
//             Self::SelfDetect(status) => status,
//             Self::Taunt(status) => status,
//             Self::OnDeath(status) => status,
//             Self::OnSpawn(status) => status,
//             Self::OnKill(status) => status,
//             Self::OnHeal(status) => status,
//             Self::OnDealDamage(status) => status,
//             Self::OnTakeDamage(status) => status,
//             Self::OnShieldBroken(status) => status,
//             Self::GainedEffect(status) => status,
//             Self::Scavenge(status) => status,
//             Self::AttackSpeed(status) => status,
//             Self::RepeatingEffect(status) => status,
//             Self::Charmed(status) => status,
//             Self::Bleed(status) => status,
//             Self::Plague(status) => status,
//             Self::SiphonLife(status) => status,
//         }
//     }
//     pub fn r#type(&self) -> StatusType {
//         match self {
//             Self::Freeze(status) => StatusType::Freeze,
//             Self::Stun(status) => StatusType::Stun,
//             Self::Shield(status) => StatusType::Shield,
//             Self::Vulnerability(status) => StatusType::Vulnerability,
//             Self::Invulnerability(status) => StatusType::Invulnerability,
//             Self::Slow(status) => StatusType::Slow,
//             Self::Modifier(status) => StatusType::Modifier,
//             Self::Aura(status) => StatusType::Aura,
//             Self::Protection(status) => StatusType::Protection,
//             Self::Detect(status) => StatusType::Detect,
//             Self::SelfDetect(status) => StatusType::SelfDetect,
//             Self::Taunt(status) => StatusType::Taunt,
//             Self::OnDeath(status) => StatusType::OnDeath,
//             Self::OnSpawn(status) => StatusType::OnSpawn,
//             Self::OnKill(status) => StatusType::OnKill,
//             Self::OnHeal(status) => StatusType::OnHeal,
//             Self::OnDealDamage(status) => StatusType::OnDealDamage,
//             Self::OnTakeDamage(status) => StatusType::OnTakeDamage,
//             Self::OnShieldBroken(status) => StatusType::OnShieldBroken,
//             Self::GainedEffect(status) => StatusType::GainedEffect,
//             Self::Scavenge(status) => StatusType::Scavenge,
//             Self::AttackSpeed(status) => StatusType::AttackSpeed,
//             Self::RepeatingEffect(status) => StatusType::RepeatingEffect,
//             Self::Charmed(status) => StatusType::Charmed,
//             Self::Bleed(status) => StatusType::Bleed,
//             Self::Plague(status) => StatusType::Plague,
//             Self::SiphonLife(status) => StatusType::SiphonLife,
//         }
//     }
//     pub fn walk_effects_mut(&mut self, f: &mut dyn FnMut(&mut Effect)) {
//         self.as_mut().walk_effects_mut(f);
//     }
// }
