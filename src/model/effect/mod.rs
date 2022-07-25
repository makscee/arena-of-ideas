use super::*;

mod action;
mod add_global_var;
mod add_targets;
mod add_var;
mod aoe;
mod apply_gained;
mod attach_status;
mod chain;
mod change_context;
mod change_context_status;
mod change_stat;
mod change_target;
mod custom_trigger;
mod damage;
mod heal;
mod if_effect;
mod instant_action;
mod list;
mod maybe_modify;
mod next_action_modifier;
mod noop;
mod random;
mod remove_status;
mod repeat;
mod revive;
mod spawn;
mod suicide;
mod visual;
mod visual_chain;

pub use action::*;
pub use add_global_var::*;
pub use add_targets::*;
pub use add_var::*;
pub use aoe::*;
pub use apply_gained::*;
pub use attach_status::*;
pub use chain::*;
pub use change_context::*;
pub use change_context_status::*;
pub use change_stat::*;
pub use change_target::*;
pub use custom_trigger::*;
pub use damage::*;
pub use heal::*;
pub use if_effect::*;
pub use instant_action::*;
pub use list::*;
pub use maybe_modify::*;
pub use next_action_modifier::*;
pub use noop::*;
pub use random::*;
pub use remove_status::*;
pub use repeat::*;
pub use revive::*;
pub use spawn::*;
pub use suicide::*;
pub use visual::*;
pub use visual_chain::*;

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type", deny_unknown_fields, from = "EffectConfig")]
pub enum Effect {
    Noop(Box<NoopEffect>),
    InstantAction(Box<InstantActionEffect>),
    Damage(Box<DamageEffect>),
    AttachStatus(Box<AttachStatusEffect>),
    Spawn(Box<SpawnEffect>),
    AOE(Box<AoeEffect>),
    Suicide(Box<SuicideEffect>),
    Chain(Box<ChainEffect>),
    AddTargets(Box<AddTargetsEffect>),
    Repeat(Box<RepeatEffect>),
    Random(Box<RandomEffect>),
    List(Box<ListEffect>),
    If(Box<IfEffect>),
    MaybeModify(Box<MaybeModifyEffect>),
    ChangeContext(Box<ChangeContextEffect>),
    ChangeContextStatus(Box<ChangeContextStatusEffect>),
    Heal(Box<HealEffect>),
    Revive(Box<ReviveEffect>),
    ApplyGained(Box<ApplyGainedEffect>),
    ChangeTarget(Box<ChangeTargetEffect>),
    ChangeStat(Box<ChangeStatEffect>),
    Action(Box<ActionEffect>),
    NextActionModifier(Box<NextActionModifierEffect>),
    Visual(Box<VisualEffect>),
    VisualChain(Box<VisualChainEffect>),
    AddVar(Box<AddVarEffect>),
    AddGlobalVar(Box<AddGlobalVarEffect>),
    RemoveStatus(Box<RemoveStatusEffect>),
    CustomTrigger(Box<CustomTriggerEffect>),
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum RawEffect {
    Noop(Box<NoopEffect>),
    InstantAction(Box<InstantActionEffect>),
    Damage(Box<DamageEffect>),
    AttachStatus(Box<AttachStatusEffect>),
    Spawn(Box<SpawnEffect>),
    AOE(Box<AoeEffect>),
    Suicide(Box<SuicideEffect>),
    Chain(Box<ChainEffect>),
    AddTargets(Box<AddTargetsEffect>),
    Repeat(Box<RepeatEffect>),
    Random(Box<RandomEffect>),
    List(Box<ListEffect>),
    If(Box<IfEffect>),
    MaybeModify(Box<MaybeModifyEffect>),
    ChangeContext(Box<ChangeContextEffect>),
    ChangeContextStatus(Box<ChangeContextStatusEffect>),
    Heal(Box<HealEffect>),
    Revive(Box<ReviveEffect>),
    ApplyGained(Box<ApplyGainedEffect>),
    ChangeTarget(Box<ChangeTargetEffect>),
    ChangeStat(Box<ChangeStatEffect>),
    Action(Box<ActionEffect>),
    NextActionModifier(Box<NextActionModifierEffect>),
    Visual(Box<VisualEffect>),
    VisualChain(Box<VisualChainEffect>),
    AddVar(Box<AddVarEffect>),
    AddGlobalVar(Box<AddGlobalVarEffect>),
    RemoveStatus(Box<RemoveStatusEffect>),
    CustomTrigger(Box<CustomTriggerEffect>),
}

#[derive(Serialize, Deserialize, Clone)]
struct EffectPreset {
    pub preset: String,
    #[serde(flatten)]
    pub overrides: serde_json::Map<String, serde_json::Value>,
}

#[derive(Serialize, Clone)]
#[serde(untagged)]
enum EffectConfig {
    Effect(RawEffect),
    Preset(EffectPreset),
}

// Implement deserialize manually for better error description
impl<'de> Deserialize<'de> for EffectConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        match EffectPreset::deserialize(value.clone()) {
            Ok(preset) => return Ok(Self::Preset(preset)),
            Err(_) => {}
        }
        let effect =
            RawEffect::deserialize(value).map_err(|error| serde::de::Error::custom(error))?;
        Ok(Self::Effect(effect))
    }
}

impl Default for EffectConfig {
    fn default() -> Self {
        Self::Effect(RawEffect::default())
    }
}

impl std::fmt::Debug for Effect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Noop(effect) => effect.fmt(f),
            Self::InstantAction(effect) => effect.fmt(f),
            Self::Damage(effect) => effect.fmt(f),
            Self::AttachStatus(effect) => effect.fmt(f),
            Self::Spawn(effect) => effect.fmt(f),
            Self::AOE(effect) => effect.fmt(f),
            Self::Suicide(effect) => effect.fmt(f),
            Self::Chain(effect) => effect.fmt(f),
            Self::AddTargets(effect) => effect.fmt(f),
            Self::Repeat(effect) => effect.fmt(f),
            Self::Random(effect) => effect.fmt(f),
            Self::List(effect) => effect.fmt(f),
            Self::If(effect) => effect.fmt(f),
            Self::MaybeModify(effect) => effect.fmt(f),
            Self::ChangeContext(effect) => effect.fmt(f),
            Self::ChangeContextStatus(effect) => effect.fmt(f),
            Self::Heal(effect) => effect.fmt(f),
            Self::Revive(effect) => effect.fmt(f),
            Self::ApplyGained(effect) => effect.fmt(f),
            Self::ChangeTarget(effect) => effect.fmt(f),
            Self::ChangeStat(effect) => effect.fmt(f),
            Self::Action(effect) => effect.fmt(f),
            Self::NextActionModifier(effect) => effect.fmt(f),
            Self::Visual(effect) => effect.fmt(f),
            Self::VisualChain(effect) => effect.fmt(f),
            Self::AddVar(effect) => effect.fmt(f),
            Self::AddGlobalVar(effect) => effect.fmt(f),
            Self::RemoveStatus(effect) => effect.fmt(f),
            Self::CustomTrigger(effect) => effect.fmt(f),
        }
    }
}

impl From<RawEffect> for Effect {
    fn from(effect: RawEffect) -> Self {
        match effect {
            RawEffect::Noop(effect) => Self::Noop(effect),
            RawEffect::InstantAction(effect) => Self::InstantAction(effect),
            RawEffect::Damage(effect) => Self::Damage(effect),
            RawEffect::AttachStatus(effect) => Self::AttachStatus(effect),
            RawEffect::Spawn(effect) => Self::Spawn(effect),
            RawEffect::AOE(effect) => Self::AOE(effect),
            RawEffect::Suicide(effect) => Self::Suicide(effect),
            RawEffect::Chain(effect) => Self::Chain(effect),
            RawEffect::AddTargets(effect) => Self::AddTargets(effect),
            RawEffect::Repeat(effect) => Self::Repeat(effect),
            RawEffect::Random(effect) => Self::Random(effect),
            RawEffect::List(effect) => Self::List(effect),
            RawEffect::If(effect) => Self::If(effect),
            RawEffect::MaybeModify(effect) => Self::MaybeModify(effect),
            RawEffect::ChangeContext(effect) => Self::ChangeContext(effect),
            RawEffect::ChangeContextStatus(effect) => Self::ChangeContextStatus(effect),
            RawEffect::Heal(effect) => Self::Heal(effect),
            RawEffect::Revive(effect) => Self::Revive(effect),
            RawEffect::ApplyGained(effect) => Self::ApplyGained(effect),
            RawEffect::ChangeTarget(effect) => Self::ChangeTarget(effect),
            RawEffect::ChangeStat(effect) => Self::ChangeStat(effect),
            RawEffect::Action(effect) => Self::Action(effect),
            RawEffect::NextActionModifier(effect) => Self::NextActionModifier(effect),
            RawEffect::Visual(effect) => Self::Visual(effect),
            RawEffect::VisualChain(effect) => Self::VisualChain(effect),
            RawEffect::AddVar(effect) => Self::AddVar(effect),
            RawEffect::AddGlobalVar(effect) => Self::AddGlobalVar(effect),
            RawEffect::RemoveStatus(effect) => Self::RemoveStatus(effect),
            RawEffect::CustomTrigger(effect) => Self::CustomTrigger(effect),
        }
    }
}

impl Default for RawEffect {
    fn default() -> Self {
        Self::noop()
    }
}

impl RawEffect {
    pub fn noop() -> Self {
        Self::Noop(Box::new(NoopEffect {}))
    }
}

impl Default for Effect {
    fn default() -> Self {
        Self::noop()
    }
}

impl Effect {
    pub fn noop() -> Self {
        RawEffect::noop().into()
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
            Effect::Damage(effect) => &mut **effect,
            Effect::AttachStatus(effect) => &mut **effect,
            Effect::Spawn(effect) => &mut **effect,
            Effect::AOE(effect) => &mut **effect,
            Effect::Suicide(effect) => &mut **effect,
            Effect::Chain(effect) => &mut **effect,
            Effect::AddTargets(effect) => &mut **effect,
            Effect::Repeat(effect) => &mut **effect,
            Effect::Random(effect) => &mut **effect,
            Effect::List(effect) => &mut **effect,
            Effect::If(effect) => &mut **effect,
            Effect::MaybeModify(effect) => &mut **effect,
            Effect::ChangeContext(effect) => &mut **effect,
            Effect::ChangeContextStatus(effect) => &mut **effect,
            Effect::Heal(effect) => &mut **effect,
            Effect::Revive(effect) => &mut **effect,
            Effect::ApplyGained(effect) => &mut **effect,
            Effect::ChangeTarget(effect) => &mut **effect,
            Effect::ChangeStat(effect) => &mut **effect,
            Effect::Action(effect) => &mut **effect,
            Effect::NextActionModifier(effect) => &mut **effect,
            Effect::Visual(effect) => &mut **effect,
            Effect::VisualChain(effect) => &mut **effect,
            Effect::AddVar(effect) => &mut **effect,
            Effect::AddGlobalVar(effect) => &mut **effect,
            Effect::RemoveStatus(effect) => &mut **effect,
            Effect::CustomTrigger(effect) => &mut **effect,
        }
    }
    pub fn as_box(self) -> Box<dyn EffectImpl> {
        match self {
            Effect::Noop(effect) => effect,
            Effect::InstantAction(effect) => effect,
            Effect::Damage(effect) => effect,
            Effect::AttachStatus(effect) => effect,
            Effect::Spawn(effect) => effect,
            Effect::AOE(effect) => effect,
            Effect::Suicide(effect) => effect,
            Effect::Chain(effect) => effect,
            Effect::AddTargets(effect) => effect,
            Effect::Repeat(effect) => effect,
            Effect::Random(effect) => effect,
            Effect::List(effect) => effect,
            Effect::If(effect) => effect,
            Effect::MaybeModify(effect) => effect,
            Effect::ChangeContext(effect) => effect,
            Effect::ChangeContextStatus(effect) => effect,
            Effect::Heal(effect) => effect,
            Effect::Revive(effect) => effect,
            Effect::ApplyGained(effect) => effect,
            Effect::ChangeTarget(effect) => effect,
            Effect::ChangeStat(effect) => effect,
            Effect::Action(effect) => effect,
            Effect::NextActionModifier(effect) => effect,
            Effect::Visual(effect) => effect,
            Effect::VisualChain(effect) => effect,
            Effect::AddVar(effect) => effect,
            Effect::AddGlobalVar(effect) => effect,
            Effect::RemoveStatus(effect) => effect,
            Effect::CustomTrigger(effect) => effect,
        }
    }
    pub fn walk_mut(&mut self, mut f: &mut dyn FnMut(&mut Effect)) {
        self.as_mut().walk_effects_mut(f);
        f(self);
    }
}

impl From<EffectConfig> for Effect {
    fn from(config: EffectConfig) -> Self {
        match config {
            EffectConfig::Effect(effect) => effect.into(),
            EffectConfig::Preset(preset) => preset.into(),
        }
    }
}

impl From<EffectPreset> for Effect {
    fn from(mut effect: EffectPreset) -> Self {
        let mut preset_json = {
            // Acquire the lock and drop it early to prevent deadlock
            let presets = EFFECT_PRESETS.lock().unwrap();
            let preset = presets.get(&effect.preset).expect(&format!(
                "Failed to find a preset effect: {}",
                effect.preset
            ));
            serde_json::to_value(preset).unwrap()
        };
        preset_json
            .as_object_mut()
            .unwrap()
            .append(&mut effect.overrides);
        // Caution: be ware of a deadlock possibility, as Effect parser uses EFFECT_PRESETS
        let effect: RawEffect =
            serde_json::from_value(preset_json).expect("Failed to override fields of the preset");
        effect.into()
    }
}
