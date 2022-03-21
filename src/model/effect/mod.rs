use super::*;

mod add_targets;
mod aoe;
mod attach_status;
mod chain;
mod change_context;
mod damage;
mod heal;
mod if_effect;
mod instant_action;
mod list;
mod maybe_modify;
mod noop;
mod projectile;
mod random;
mod repeat;
mod spawn;
mod suicide;
mod time_bomb;

pub use add_targets::*;
pub use aoe::*;
pub use attach_status::*;
pub use chain::*;
pub use change_context::*;
pub use damage::*;
pub use heal::*;
pub use if_effect::*;
pub use instant_action::*;
pub use list::*;
pub use maybe_modify::*;
pub use noop::*;
pub use projectile::*;
pub use random::*;
pub use repeat::*;
pub use spawn::*;
pub use suicide::*;
pub use time_bomb::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
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
    AddTargets(Box<AddTargetsEffect>),
    Repeat(Box<RepeatEffect>),
    Random(Box<RandomEffect>),
    List(Box<ListEffect>),
    If(Box<IfEffect>),
    MaybeModify(Box<MaybeModifyEffect>),
    ChangeContext(Box<ChangeContextEffect>),
    Heal(Box<HealEffect>),
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
            Effect::AddTargets(effect) => &mut **effect,
            Effect::Repeat(effect) => &mut **effect,
            Effect::Random(effect) => &mut **effect,
            Effect::List(effect) => &mut **effect,
            Effect::If(effect) => &mut **effect,
            Effect::MaybeModify(effect) => &mut **effect,
            Effect::ChangeContext(effect) => &mut **effect,
            Effect::Heal(effect) => &mut **effect,
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
            Effect::AddTargets(effect) => effect,
            Effect::Repeat(effect) => effect,
            Effect::Random(effect) => effect,
            Effect::List(effect) => effect,
            Effect::If(effect) => effect,
            Effect::MaybeModify(effect) => effect,
            Effect::ChangeContext(effect) => effect,
            Effect::Heal(effect) => effect,
        }
    }
    pub fn walk_mut(&mut self, mut f: &mut dyn FnMut(&mut Effect)) {
        self.as_mut().walk_effects_mut(f);
        f(self);
    }
}
