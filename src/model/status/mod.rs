use super::*;

mod attack_speed;
mod aura;
mod detect;
mod freeze;
mod gained_effect;
mod invulnerability;
mod modifier;
mod on_death;
mod on_heal;
mod on_kill;
mod on_shield_broken;
mod on_spawn;
mod on_take_damage;
mod protection;
mod scavenge;
mod shield;
mod slow;
mod stun;
mod taunt;

pub use attack_speed::*;
pub use aura::*;
pub use detect::*;
pub use freeze::*;
pub use gained_effect::*;
pub use invulnerability::*;
pub use modifier::*;
pub use on_death::*;
pub use on_heal::*;
pub use on_kill::*;
pub use on_shield_broken::*;
pub use on_spawn::*;
pub use on_take_damage::*;
pub use protection::*;
pub use scavenge::*;
pub use shield::*;
pub use slow::*;
pub use stun::*;
pub use taunt::*;

#[derive(Debug, Serialize, Deserialize, Copy, Clone, Eq, PartialEq, Hash)]
pub enum StatusType {
    Freeze,
    Stun,
    Shield,
    Invulnerability,
    Slow,
    Modifier,
    Aura,
    Protection,
    Detect,
    Taunt,
    OnDeath,
    OnSpawn,
    OnKill,
    OnHeal,
    OnTakeDamage,
    OnShieldBroken,
    GainedEffect,
    Scavenge,
    AttackSpeed,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum Status {
    Freeze(Box<FreezeStatus>),
    Stun(Box<StunStatus>),
    Shield(Box<ShieldStatus>),
    Invulnerability(Box<InvulnerabilityStatus>),
    Slow(Box<SlowStatus>),
    Modifier(Box<ModifierStatus>),
    Aura(Box<AuraStatus>),
    Protection(Box<ProtectionStatus>),
    Detect(Box<DetectStatus>),
    Taunt(Box<TauntStatus>),
    OnDeath(Box<OnDeathStatus>),
    OnSpawn(Box<OnSpawnStatus>),
    OnKill(Box<OnKillStatus>),
    OnHeal(Box<OnHealStatus>),
    OnTakeDamage(Box<OnTakeDamageStatus>),
    OnShieldBroken(Box<OnShieldBrokenStatus>),
    GainedEffect(Box<GainedEffectStatus>),
    Scavenge(Box<ScavengeStatus>),
    AttackSpeed(Box<AttackSpeedStatus>),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AttachedStatus {
    #[serde(flatten)]
    pub status: Status,
    pub time: Option<Time>,
}

pub trait StatusImpl: EffectContainer {}

impl Status {
    pub fn as_mut(&mut self) -> &mut dyn StatusImpl {
        match self {
            Self::Freeze(status) => &mut **status,
            Self::Stun(status) => &mut **status,
            Self::Shield(status) => &mut **status,
            Self::Invulnerability(status) => &mut **status,
            Self::Slow(status) => &mut **status,
            Self::Modifier(status) => &mut **status,
            Self::Aura(status) => &mut **status,
            Self::Protection(status) => &mut **status,
            Self::Detect(status) => &mut **status,
            Self::Taunt(status) => &mut **status,
            Self::OnDeath(status) => &mut **status,
            Self::OnSpawn(status) => &mut **status,
            Self::OnKill(status) => &mut **status,
            Self::OnHeal(status) => &mut **status,
            Self::OnTakeDamage(status) => &mut **status,
            Self::OnShieldBroken(status) => &mut **status,
            Self::GainedEffect(status) => &mut **status,
            Self::Scavenge(status) => &mut **status,
            Self::AttackSpeed(status) => &mut **status,
        }
    }
    pub fn as_box(self) -> Box<dyn StatusImpl> {
        match self {
            Self::Freeze(status) => status,
            Self::Stun(status) => status,
            Self::Shield(status) => status,
            Self::Invulnerability(status) => status,
            Self::Slow(status) => status,
            Self::Modifier(status) => status,
            Self::Aura(status) => status,
            Self::Protection(status) => status,
            Self::Detect(status) => status,
            Self::Taunt(status) => status,
            Self::OnDeath(status) => status,
            Self::OnSpawn(status) => status,
            Self::OnKill(status) => status,
            Self::OnHeal(status) => status,
            Self::OnTakeDamage(status) => status,
            Self::OnShieldBroken(status) => status,
            Self::GainedEffect(status) => status,
            Self::Scavenge(status) => status,
            Self::AttackSpeed(status) => status,
        }
    }
    pub fn r#type(&self) -> StatusType {
        match self {
            Self::Freeze(status) => StatusType::Freeze,
            Self::Stun(status) => StatusType::Stun,
            Self::Shield(status) => StatusType::Shield,
            Self::Invulnerability(status) => StatusType::Invulnerability,
            Self::Slow(status) => StatusType::Slow,
            Self::Modifier(status) => StatusType::Modifier,
            Self::Aura(status) => StatusType::Aura,
            Self::Protection(status) => StatusType::Protection,
            Self::Detect(status) => StatusType::Detect,
            Self::Taunt(status) => StatusType::Taunt,
            Self::OnDeath(status) => StatusType::OnDeath,
            Self::OnSpawn(status) => StatusType::OnSpawn,
            Self::OnKill(status) => StatusType::OnKill,
            Self::OnHeal(status) => StatusType::OnHeal,
            Self::OnTakeDamage(status) => StatusType::OnTakeDamage,
            Self::OnShieldBroken(status) => StatusType::OnShieldBroken,
            Self::GainedEffect(status) => StatusType::GainedEffect,
            Self::Scavenge(status) => StatusType::Scavenge,
            Self::AttackSpeed(status) => StatusType::AttackSpeed,
        }
    }
    pub fn walk_effects_mut(&mut self, f: &mut dyn FnMut(&mut Effect)) {
        self.as_mut().walk_effects_mut(f);
    }
}
