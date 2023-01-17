use super::*;

mod add_global_var;
mod add_targets;
mod add_var;
mod change_context;
mod change_stat;
mod change_target;
mod custom_trigger;
mod damage;
mod if_effect;
mod kill;
mod list;
mod message;
mod noop;
mod random;
mod repeat;
mod sound;
mod spawn;

pub use add_global_var::*;
pub use add_targets::*;
pub use add_var::*;
pub use change_context::*;
pub use change_stat::*;
pub use change_target::*;
pub use custom_trigger::*;
pub use damage::*;
pub use if_effect::*;
pub use kill::*;
pub use list::*;
pub use message::*;
pub use noop::*;
pub use random::*;
pub use repeat::*;
pub use sound::SoundEffect;
pub use spawn::*;

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type", deny_unknown_fields, from = "EffectConfig")]
pub enum LogicEffect {
    Noop(Box<NoopEffect>),
    Damage(Box<DamageEffect>),
    Spawn(Box<SpawnEffect>),
    Message(Box<MessageEffect>),
    Kill(Box<KillEffect>),
    AddTargets(Box<AddTargetsEffect>),
    Repeat(Box<RepeatEffect>),
    Random(Box<RandomEffect>),
    List(Box<ListEffect>),
    If(Box<IfEffect>),
    ChangeContext(Box<ChangeContextEffect>),
    ChangeTarget(Box<ChangeTargetEffect>),
    ChangeStat(Box<ChangeStatEffect>),
    AddVar(Box<AddVarEffect>),
    AddGlobalVar(Box<AddGlobalVarEffect>),
    CustomTrigger(Box<CustomTriggerEffect>),
    Sound(Box<SoundEffect>),
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum RawEffect {
    Noop(Box<NoopEffect>),
    Damage(Box<DamageEffect>),
    Spawn(Box<SpawnEffect>),
    Kill(Box<KillEffect>),
    AddTargets(Box<AddTargetsEffect>),
    Repeat(Box<RepeatEffect>),
    Random(Box<RandomEffect>),
    List(Box<ListEffect>),
    If(Box<IfEffect>),
    ChangeContext(Box<ChangeContextEffect>),
    Message(Box<MessageEffect>),
    ChangeTarget(Box<ChangeTargetEffect>),
    ChangeStat(Box<ChangeStatEffect>),
    AddVar(Box<AddVarEffect>),
    AddGlobalVar(Box<AddGlobalVarEffect>),
    CustomTrigger(Box<CustomTriggerEffect>),
    Sound(Box<SoundEffect>),
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
    // Preset(EffectPreset),
}

// Implement deserialize manually for better error description
impl<'de> Deserialize<'de> for EffectConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        // match EffectPreset::deserialize(value.clone()) {
        //     Ok(preset) => return Ok(Self::Preset(preset)),
        //     Err(_) => {}
        // }
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

impl std::fmt::Debug for LogicEffect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Noop(effect) => effect.fmt(f),
            Self::Damage(effect) => effect.fmt(f),
            Self::Spawn(effect) => effect.fmt(f),
            Self::Kill(effect) => effect.fmt(f),
            Self::AddTargets(effect) => effect.fmt(f),
            Self::Repeat(effect) => effect.fmt(f),
            Self::Random(effect) => effect.fmt(f),
            Self::List(effect) => effect.fmt(f),
            Self::Message(effect) => effect.fmt(f),
            Self::If(effect) => effect.fmt(f),
            Self::ChangeContext(effect) => effect.fmt(f),
            Self::ChangeTarget(effect) => effect.fmt(f),
            Self::ChangeStat(effect) => effect.fmt(f),
            Self::AddVar(effect) => effect.fmt(f),
            Self::AddGlobalVar(effect) => effect.fmt(f),
            Self::CustomTrigger(effect) => effect.fmt(f),
            Self::Sound(effect) => effect.fmt(f),
        }
    }
}

impl From<RawEffect> for LogicEffect {
    fn from(effect: RawEffect) -> Self {
        match effect {
            RawEffect::Noop(effect) => Self::Noop(effect),
            RawEffect::Damage(effect) => Self::Damage(effect),
            RawEffect::Spawn(effect) => Self::Spawn(effect),
            RawEffect::Kill(effect) => Self::Kill(effect),
            RawEffect::AddTargets(effect) => Self::AddTargets(effect),
            RawEffect::Repeat(effect) => Self::Repeat(effect),
            RawEffect::Random(effect) => Self::Random(effect),
            RawEffect::List(effect) => Self::List(effect),
            RawEffect::If(effect) => Self::If(effect),
            RawEffect::ChangeContext(effect) => Self::ChangeContext(effect),
            RawEffect::ChangeTarget(effect) => Self::ChangeTarget(effect),
            RawEffect::ChangeStat(effect) => Self::ChangeStat(effect),
            RawEffect::AddVar(effect) => Self::AddVar(effect),
            RawEffect::Message(effect) => Self::Message(effect),
            RawEffect::AddGlobalVar(effect) => Self::AddGlobalVar(effect),
            RawEffect::CustomTrigger(effect) => Self::CustomTrigger(effect),
            RawEffect::Sound(effect) => Self::Sound(effect),
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

impl Default for LogicEffect {
    fn default() -> Self {
        Self::noop()
    }
}

impl LogicEffect {
    pub fn noop() -> Self {
        RawEffect::noop().into()
    }
}

pub trait EffectContainer {
    fn walk_effects_mut(&mut self, f: &mut dyn FnMut(&mut LogicEffect)) {}
}

pub trait EffectImpl: EffectContainer {
    fn process(self: Box<Self>, context: LogicEffectContext, logic: &mut Logic);
}

impl LogicEffect {
    pub fn as_mut(&mut self) -> &mut dyn EffectImpl {
        match self {
            LogicEffect::Noop(effect) => &mut **effect,
            LogicEffect::Damage(effect) => &mut **effect,
            LogicEffect::Spawn(effect) => &mut **effect,
            LogicEffect::Kill(effect) => &mut **effect,
            LogicEffect::AddTargets(effect) => &mut **effect,
            LogicEffect::Repeat(effect) => &mut **effect,
            LogicEffect::Random(effect) => &mut **effect,
            LogicEffect::List(effect) => &mut **effect,
            LogicEffect::If(effect) => &mut **effect,
            LogicEffect::ChangeContext(effect) => &mut **effect,
            LogicEffect::Message(effect) => &mut **effect,
            LogicEffect::ChangeTarget(effect) => &mut **effect,
            LogicEffect::ChangeStat(effect) => &mut **effect,
            LogicEffect::AddVar(effect) => &mut **effect,
            LogicEffect::AddGlobalVar(effect) => &mut **effect,
            LogicEffect::CustomTrigger(effect) => &mut **effect,
            LogicEffect::Sound(effect) => &mut **effect,
        }
    }
    pub fn as_box(self) -> Box<dyn EffectImpl> {
        match self {
            LogicEffect::Noop(effect) => effect,
            LogicEffect::Damage(effect) => effect,
            LogicEffect::Spawn(effect) => effect,
            LogicEffect::Kill(effect) => effect,
            LogicEffect::AddTargets(effect) => effect,
            LogicEffect::Repeat(effect) => effect,
            LogicEffect::Random(effect) => effect,
            LogicEffect::List(effect) => effect,
            LogicEffect::If(effect) => effect,
            LogicEffect::ChangeContext(effect) => effect,
            LogicEffect::Message(effect) => effect,
            LogicEffect::ChangeTarget(effect) => effect,
            LogicEffect::ChangeStat(effect) => effect,
            LogicEffect::AddVar(effect) => effect,
            LogicEffect::AddGlobalVar(effect) => effect,
            LogicEffect::CustomTrigger(effect) => effect,
            LogicEffect::Sound(effect) => effect,
        }
    }
    pub fn walk_mut(&mut self, mut f: &mut dyn FnMut(&mut LogicEffect)) {
        self.as_mut().walk_effects_mut(f);
        f(self);
    }
}

impl From<EffectConfig> for LogicEffect {
    fn from(config: EffectConfig) -> Self {
        match config {
            EffectConfig::Effect(effect) => effect.into(),
            // EffectConfig::Preset(preset) => preset.into(),
        }
    }
}

// todo: reimplement
// impl From<EffectPreset> for LogicEffect {
//     fn from(mut effect: EffectPreset) -> Self {
//         let mut preset_json = {
//             // Acquire the lock and drop it early to prevent deadlock
//             let presets = EFFECT_PRESETS.lock().unwrap();
//             let preset = presets.get(&effect.preset).expect(&format!(
//                 "Failed to find a preset effect: {}",
//                 effect.preset
//             ));
//             serde_json::to_value(preset).unwrap()
//         };
//         preset_json
//             .as_object_mut()
//             .unwrap()
//             .append(&mut effect.overrides);
//         // Caution: be ware of a deadlock possibility, as Effect parser uses EFFECT_PRESETS
//         let effect: RawEffect =
//             serde_json::from_value(preset_json).expect("Failed to override fields of the preset");
//         effect.into()
//     }
// }
