use super::*;

mod action;
mod add_targets;
mod aoe;
mod apply_gained;
mod attach_status;
mod chain;
mod change_context;
mod change_stat;
mod change_target;
mod damage;
mod delayed;
mod glave;
mod heal;
mod if_effect;
mod instant_action;
mod list;
mod maybe_modify;
mod next_action_modifier;
mod noop;
mod projectile;
mod random;
mod repeat;
mod revive;
mod spawn;
mod splash;
mod suicide;
mod time_bomb;

pub use action::*;
pub use add_targets::*;
pub use aoe::*;
pub use apply_gained::*;
pub use attach_status::*;
pub use chain::*;
pub use change_context::*;
pub use change_stat::*;
pub use change_target::*;
pub use damage::*;
pub use delayed::*;
pub use glave::*;
pub use heal::*;
pub use if_effect::*;
pub use instant_action::*;
pub use list::*;
pub use maybe_modify::*;
pub use next_action_modifier::*;
pub use noop::*;
pub use projectile::*;
pub use random::*;
pub use repeat::*;
pub use revive::*;
pub use spawn::*;
pub use splash::*;
pub use suicide::*;
pub use time_bomb::*;

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum Effect {
    Noop(Box<NoopEffect>),
    InstantAction(Box<InstantActionEffect>),
    Projectile(Box<ProjectileEffect>),
    Damage(Box<DamageEffect>),
    AttachStatus(Box<AttachStatusEffect>),
    Spawn(Box<SpawnEffect>),
    AOE(Box<AoeEffect>),
    TimeBomb(Box<TimeBombEffect>),
    Suicide(Box<SuicideEffect>),
    Chain(Box<ChainEffect>),
    Glave(Box<GlaveEffect>),
    AddTargets(Box<AddTargetsEffect>),
    Repeat(Box<RepeatEffect>),
    Random(Box<RandomEffect>),
    List(Box<ListEffect>),
    If(Box<IfEffect>),
    MaybeModify(Box<MaybeModifyEffect>),
    ChangeContext(Box<ChangeContextEffect>),
    Heal(Box<HealEffect>),
    Revive(Box<ReviveEffect>),
    ApplyGained(Box<ApplyGainedEffect>),
    ChangeTarget(Box<ChangeTargetEffect>),
    ChangeStat(Box<ChangeStatEffect>),
    Delayed(Box<DelayedEffect>),
    Action(Box<ActionEffect>),
    Splash(Box<SplashEffect>),
    NextActionModifier(Box<NextActionModifierEffect>),
}

impl std::fmt::Debug for Effect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Noop(effect) => effect.fmt(f),
            Self::InstantAction(effect) => effect.fmt(f),
            Self::Projectile(effect) => effect.fmt(f),
            Self::Damage(effect) => effect.fmt(f),
            Self::AttachStatus(effect) => effect.fmt(f),
            Self::Spawn(effect) => effect.fmt(f),
            Self::AOE(effect) => effect.fmt(f),
            Self::TimeBomb(effect) => effect.fmt(f),
            Self::Suicide(effect) => effect.fmt(f),
            Self::Chain(effect) => effect.fmt(f),
            Self::Glave(effect) => effect.fmt(f),
            Self::AddTargets(effect) => effect.fmt(f),
            Self::Repeat(effect) => effect.fmt(f),
            Self::Random(effect) => effect.fmt(f),
            Self::List(effect) => effect.fmt(f),
            Self::If(effect) => effect.fmt(f),
            Self::MaybeModify(effect) => effect.fmt(f),
            Self::ChangeContext(effect) => effect.fmt(f),
            Self::Heal(effect) => effect.fmt(f),
            Self::Revive(effect) => effect.fmt(f),
            Self::ApplyGained(effect) => effect.fmt(f),
            Self::ChangeTarget(effect) => effect.fmt(f),
            Self::ChangeStat(effect) => effect.fmt(f),
            Self::Delayed(effect) => effect.fmt(f),
            Self::Action(effect) => effect.fmt(f),
            Self::Splash(effect) => effect.fmt(f),
            Self::NextActionModifier(effect) => effect.fmt(f),
        }
    }
}

impl Default for Effect {
    fn default() -> Self {
        Self::noop()
    }
}

impl Effect {
    pub fn noop() -> Self {
        Self::Noop(Box::new(NoopEffect {}))
    }
}

pub trait EffectContainer {
    fn walk_effects_mut(&mut self, f: &mut dyn FnMut(&mut Effect));
}

pub trait EffectImpl: EffectContainer {
    fn process(self: Box<Self>, context: EffectContext, logic: &mut Logic);
}

impl Effect {
    pub fn as_mut(&mut self) -> &mut dyn EffectImpl {
        match self {
            Effect::Noop(effect) => &mut **effect,
            Effect::InstantAction(effect) => &mut **effect,
            Effect::Projectile(effect) => &mut **effect,
            Effect::Damage(effect) => &mut **effect,
            Effect::AttachStatus(effect) => &mut **effect,
            Effect::Spawn(effect) => &mut **effect,
            Effect::AOE(effect) => &mut **effect,
            Effect::TimeBomb(effect) => &mut **effect,
            Effect::Suicide(effect) => &mut **effect,
            Effect::Chain(effect) => &mut **effect,
            Effect::Glave(effect) => &mut **effect,
            Effect::AddTargets(effect) => &mut **effect,
            Effect::Repeat(effect) => &mut **effect,
            Effect::Random(effect) => &mut **effect,
            Effect::List(effect) => &mut **effect,
            Effect::If(effect) => &mut **effect,
            Effect::MaybeModify(effect) => &mut **effect,
            Effect::ChangeContext(effect) => &mut **effect,
            Effect::Heal(effect) => &mut **effect,
            Effect::Revive(effect) => &mut **effect,
            Effect::ApplyGained(effect) => &mut **effect,
            Effect::ChangeTarget(effect) => &mut **effect,
            Effect::ChangeStat(effect) => &mut **effect,
            Effect::Delayed(effect) => &mut **effect,
            Effect::Action(effect) => &mut **effect,
            Effect::Splash(effect) => &mut **effect,
            Effect::NextActionModifier(effect) => &mut **effect,
        }
    }
    pub fn as_box(self) -> Box<dyn EffectImpl> {
        match self {
            Effect::Noop(effect) => effect,
            Effect::InstantAction(effect) => effect,
            Effect::Projectile(effect) => effect,
            Effect::Damage(effect) => effect,
            Effect::AttachStatus(effect) => effect,
            Effect::Spawn(effect) => effect,
            Effect::AOE(effect) => effect,
            Effect::TimeBomb(effect) => effect,
            Effect::Suicide(effect) => effect,
            Effect::Chain(effect) => effect,
            Effect::Glave(effect) => effect,
            Effect::AddTargets(effect) => effect,
            Effect::Repeat(effect) => effect,
            Effect::Random(effect) => effect,
            Effect::List(effect) => effect,
            Effect::If(effect) => effect,
            Effect::MaybeModify(effect) => effect,
            Effect::ChangeContext(effect) => effect,
            Effect::Heal(effect) => effect,
            Effect::Revive(effect) => effect,
            Effect::ApplyGained(effect) => effect,
            Effect::ChangeTarget(effect) => effect,
            Effect::ChangeStat(effect) => effect,
            Effect::Delayed(effect) => effect,
            Effect::Action(effect) => effect,
            Effect::Splash(effect) => effect,
            Effect::NextActionModifier(effect) => effect,
        }
    }
    pub fn walk_mut(&mut self, mut f: &mut dyn FnMut(&mut Effect)) {
        self.as_mut().walk_effects_mut(f);
        f(self);
    }
}
