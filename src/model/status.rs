use super::*;

fn zero() -> R32 {
    R32::ZERO
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub enum StatusAction {
    Add,
    Remove,
}

impl Default for StatusAction {
    fn default() -> Self {
        Self::Add
    }
}

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

impl Default for StatusStacking {
    fn default() -> Self {
        Self::Independent
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum StatusTrigger {
    /// Triggered when the owner deals damage of the specified type (or any type if none is specified)
    DamageDealt {
        damage_type: Option<DamageType>,
        except: Option<DamageType>,
    },
    /// Triggered when the owner takes damage of the specified type (or any type if none is specified)
    DamageTaken {
        damage_type: Option<DamageType>,
        except: Option<DamageType>,
    },
    /// Triggered when the owner is about to take damage of the specified type (or any type if none is specified)
    DamageIncoming {
        damage_type: Option<DamageType>,
        except: Option<DamageType>,
    },
    /// Triggered when the owner is healed
    HealTaken { heal_type: Option<HealType> },
    /// Triggered when the owner heals someone
    HealDealt { heal_type: Option<HealType> },
    /// Triggered when the owner heals someone
    HealIncoming { heal_type: Option<HealType> },
    /// Triggered when the unit spawns
    Spawn,
    /// Triggered when the owner dies
    Death,
    /// Triggered when the owner kills another unit with damage of the specified type (or any if none is specified)
    Kill {
        damage_type: Option<DamageType>,
        except: Option<DamageType>,
    },
    /// Triggered when a unit dies in range
    Scavenge {
        who: TargetFilter,
        range: Coord,
        clan: Option<Clan>,
    },
    /// Triggered when the owner gains an effect via [ApplyGainedEffect]
    GainedEffect,
    /// Triggered when some unit acquires the specified status and the filter is satisfied
    DetectAttach {
        status_name: StatusName,
        #[serde(default)]
        status_action: StatusAction,
        filter: TargetFilter,
    },
    /// Triggered when the owner acquires the specified status
    SelfDetectAttach {
        status_name: StatusName,
        #[serde(default)]
        status_action: StatusAction,
    },
    /// Triggered periodically
    Repeating {
        #[serde(default = "zero")]
        tick_time: RealImpl<f32>,
        #[serde(default = "zero")]
        next_tick: RealImpl<f32>,
        //Utility field
        #[serde(default = "zero")]
        last_tick: RealImpl<f32>,
    },
    /// Triggered by CustomTriggerEffect
    Custom { name: String },
    /// Triggered right after status was attached
    Init,
    /// Triggered right after status was detached
    Break,
    /// Triggered when unit uses Action
    Action,
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

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum UnitStatFlag {
    MoveUnable,
    ActionUnable,
    AbilityUnable,
    DamageImmune,
    HealingImmune,
    AttachStatusImmune,
}

/// Refers to a status either by name or directly
#[derive(Debug, Serialize, Clone)]
#[serde(untagged)]
pub enum StatusRef {
    Name(StatusName),
    Raw(Status),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Aura {
    /// If specified, the aura affects only units in that radius.
    /// Otherwise, affects all units
    pub radius: Option<Coord>,
    /// Filter units by clans
    #[serde(default)]
    pub filter: ClanFilter,
    /// Additional conditional filter for units
    #[serde(default)]
    pub condition: Condition,
    /// The statuses that will be attached to the affected units
    #[serde(default)]
    pub statuses: Vec<StatusRef>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum ModifierTarget {
    Stat {
        stat: UnitStat,
        value: Expr,
    },
    ExtraOutDamageType {
        source: HashSet<DamageType>,
        damage_type: HashSet<DamageType>,
    },
    Damage {
        source: Option<HashSet<DamageType>>,
        condition: Option<Condition>,
        value: Expr,
    },
    List {
        targets: Vec<ModifierTarget>,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct StatusModifier {
    /// Specifies what the modifier effect will actually modify
    pub target: ModifierTarget,
    /// Lower priority modifiers get processed earlier
    pub priority: i64,
    /// Condition when to apply modifier
    pub condition: Condition,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum StatusEffect {
    Status,
    Aura(Aura),
    Modifier(StatusModifier),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Status {
    /// The name is used when comparing two statuses for equality for a stack
    /// and for parsing in the unit config
    pub name: StatusName,
    #[serde(flatten)]
    pub effect: StatusEffect,
    #[serde(default)]
    pub stacking: StatusStacking,
    /// While the status is active, these flags are assigned to the owner
    #[serde(default)]
    pub flags: Vec<UnitStatFlag>,
    /// If specified, the status will drop after that time,
    /// otherwise the status will be attached indefinitely
    /// or until it gets removed manually
    pub duration: Option<Time>,
    /// Specifications of effects to apply for different subsets of triggers
    #[serde(default)]
    pub listeners: Vec<StatusListener>,
    /// Initial variables
    #[serde(default)]
    pub vars: HashMap<VarName, R32>,
    #[serde(default)]
    pub order: i32,
    #[serde(skip, default = "Status::default_color")]
    pub color: Color<f32>,
    /// Whether the status will be hidden in status description render
    #[serde(default = "Status::default_hidden")]
    pub hidden: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct AttachedStatus {
    /// The actual status that hold all the neccessary logic info
    pub status: Status,
    /// Whether this status originated from an aura
    pub is_aura: bool,
    /// Whether trigger Init was fired
    pub is_inited: bool,
    /// Specifies how much time is left until the status is dropped.
    /// If `None`, then the status remains attached.
    pub time: Option<Time>,
    /// Specifies the owner of the status
    pub owner: Option<Id>,
    /// Specifies the caster that applied the status
    pub caster: Option<Id>,
    /// Variables that persist for the lifetime of the status
    pub vars: HashMap<VarName, R32>,
    pub id: Id,
}

impl StatusRef {
    pub fn name(&self) -> &StatusName {
        match self {
            StatusRef::Name(name) => name,
            StatusRef::Raw(status) => &status.name,
        }
    }

    pub fn get<'a>(&'a self, statuses: &'a Statuses) -> &'a Status {
        match self {
            StatusRef::Name(name) => {
                &statuses
                    .get(name)
                    .expect(&format!("Failed to find status {name:?}"))
                    .status
            }
            StatusRef::Raw(status) => status,
        }
    }
}

impl Status {
    /// Transforms config into an attached status
    pub fn attach(self, owner: Option<Id>, caster: Option<Id>, next_id: &mut Id) -> AttachedStatus {
        let id = *next_id;
        *next_id += 1;
        AttachedStatus {
            vars: self.vars.clone(),
            time: self.duration,
            is_aura: false,
            is_inited: false,
            status: self,
            owner,
            caster,
            id,
        }
    }

    /// Transforms config into an attached status with `is_aura` set to true
    /// and `time` set to 0
    pub fn attach_aura(self, owner: Option<Id>, caster: Id) -> AttachedStatus {
        AttachedStatus {
            vars: self.vars.clone(),
            time: Some(Time::ZERO),
            is_aura: true,
            is_inited: false,
            status: self,
            caster: Some(caster),
            owner,
            id: -1,
        }
    }

    fn default_color() -> Color<f32> {
        Color::WHITE
    }

    fn default_hidden() -> bool {
        false
    }
}

impl AttachedStatus {
    /// Reacts to the trigger and returns the relevant effects
    /// according to the status listeners
    pub fn trigger<'a>(
        &'a self,
        mut filter: impl FnMut(&StatusTrigger) -> bool + 'a,
    ) -> impl Iterator<Item = (Effect, HashMap<VarName, R32>, Id, Color<f32>)> + 'a {
        self.status
            .listeners
            .iter()
            .filter(move |listener| listener.triggers.iter().any(|trigger| filter(trigger)))
            .map(|listener| {
                (
                    listener.effect.clone(),
                    self.vars.clone(),
                    self.id,
                    self.status.color,
                )
            })
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

pub fn unit_attach_status(
    mut status: AttachedStatus,
    all_statuses: &mut Vec<AttachedStatus>,
) -> Id {
    fn replace(
        status: AttachedStatus,
        all_statuses: &mut Vec<AttachedStatus>,
        update: impl FnOnce(&mut AttachedStatus) -> Id,
    ) -> Id {
        match all_statuses
            .iter_mut()
            .find(|s| s.status.name == status.status.name)
        {
            Some(status) => {
                return update(status);
            }
            None => {
                let id = status.id;
                all_statuses.push(status);
                return id;
            }
        }
    }

    status.vars.insert(VarName::StackCounter, r32(1.0));
    match &status.status.stacking {
        StatusStacking::Independent => {
            let id = status.id;
            all_statuses.push(status);
            return id;
        }
        StatusStacking::Refresh => {
            return replace(status, all_statuses, |s| {
                s.time = s.status.duration;
                s.id
            })
        }
        StatusStacking::Count => {
            return replace(status, all_statuses, |s| {
                *s.vars.entry(VarName::StackCounter).or_insert(R32::ZERO) += r32(1.0);
                s.id
            })
        }
        StatusStacking::CountRefresh => {
            return replace(status, all_statuses, |s| {
                s.time = s.status.duration;
                *s.vars.entry(VarName::StackCounter).or_insert(R32::ZERO) += r32(1.0);
                s.id
            })
        }
    }
}

// Implement deserialize manually for better error description
impl<'de> Deserialize<'de> for StatusRef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        match StatusName::deserialize(value.clone()) {
            Ok(preset) => return Ok(Self::Name(preset)),
            Err(_) => {}
        }
        let effect = Status::deserialize(value).map_err(|error| serde::de::Error::custom(error))?;
        Ok(Self::Raw(effect))
    }
}
