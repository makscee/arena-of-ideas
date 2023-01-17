use super::*;

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
    /// Triggered when any unit deals damage of the specified type (or any type if none is specified)
    DamageHits {
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
    /// Triggered when the unit spawns
    Spawn,
    /// Triggered when the owner dies
    Death,
    /// Triggered when any unit of faction dies (or any faction if none is specified)
    DetectDeath { condition: Condition },
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
    /// Triggered by CustomTriggerEffect
    Custom { name: String },
    /// Triggered right after status was attached
    Init,
    /// Triggered right after status was detached
    Break,
    /// Triggered the moment before unit attacks
    PreStrike,
    /// Triggered the moment after unit attacks
    PostStrike,
}

impl fmt::Display for StatusTrigger {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
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
pub struct Aura {
    /// If specified, the aura affects only units in that radius.
    /// Otherwise, affects all units
    pub radius: Option<Coord>,
    /// Additional conditional filter for units
    #[serde(default)]
    pub condition: Condition,
    /// The statuses that will be attached to the affected units
    #[serde(default)]
    pub statuses: Vec<Status>,
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
    #[serde(default)]
    pub priority: i64,
    /// Condition when to apply modifier
    pub condition: Option<Condition>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum StatusEffect {
    Status,
    Aura(Aura),
    Modifier(StatusModifier),
}

pub struct TriggerEffect {
    pub status_id: Id,
    pub status_color: Rgba<f32>,
    pub effect: Effect,
    pub trigger: StatusTrigger,
    pub vars: HashMap<VarName, i32>,
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
    /// If specified, the status will drop after that time,
    /// otherwise the status will be attached indefinitely
    /// or until it gets removed manually
    pub duration: Option<std::num::NonZeroU64>,
    /// Specifications of effects to apply for different subsets of triggers
    #[serde(default)]
    pub listeners: Vec<StatusListener>,
    /// Initial variables
    #[serde(default)]
    pub vars: HashMap<VarName, i32>,
    #[serde(default)]
    pub order: i32,
    #[serde(skip, default = "Status::default_color")]
    pub color: Rgba<f32>,
    /// Whether the status will be hidden in status description render
    #[serde(default = "Status::default_hidden")]
    pub hidden: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct AttachedStatus {
    /// The actual status that hold all the neccessary logic info
    pub status: Status,
    /// Whether this status originated from an aura.
    /// If it did, the aura's Id is given
    pub is_aura: Option<Id>,
    /// Whether trigger Init was fired
    pub is_inited: bool,
    /// Specifies the owner of the status
    pub owner: Id,
    /// Specifies the creator that applied the status
    pub creator: Id,
    /// Variables that persist for the lifetime of the status
    pub vars: HashMap<VarName, i32>,
    pub id: Id,
}

impl TriggerEffect {
    pub fn new(
        status_id: Id,
        effect: Effect,
        trigger: StatusTrigger,
        vars: HashMap<VarName, i32>,
        status_color: Rgba<f32>,
    ) -> Self {
        Self {
            status_id,
            effect,
            trigger,
            vars,
            status_color,
        }
    }
}

impl Status {
    /// Transforms config into an attached status
    pub fn attach(self, owner: Id, creator: Id, id: Id) -> AttachedStatus {
        AttachedStatus {
            vars: self.vars.clone(),
            is_aura: None,
            is_inited: false,
            status: self,
            owner,
            creator,
            id,
        }
    }

    /// Transforms config into an attached status with `is_aura` set to `true`
    /// and `time` set to `None`, which means that it needs to be removed manually
    pub fn attach_aura(self, aura_id: Id, owner: Id, creator: Id) -> AttachedStatus {
        AttachedStatus {
            vars: self.vars.clone(),
            is_aura: Some(aura_id),
            is_inited: false,
            status: self,
            creator,
            owner,
            id: -1,
        }
    }

    fn default_color() -> Rgba<f32> {
        Rgba::BLACK
    }

    fn default_hidden() -> bool {
        false
    }
}

impl AttachedStatus {
    /// Reacts to the trigger and returns the relevant effects
    /// according to the status listeners
    pub fn trigger<'a>(&'a self) -> impl Iterator<Item = TriggerEffect> + 'a {
        self.status.listeners.iter().flat_map(move |listener| {
            listener.triggers.iter().map(|trigger| {
                TriggerEffect::new(
                    self.id,
                    listener.effect.clone(),
                    trigger.clone(),
                    self.vars.clone(),
                    self.status.color,
                )
            })
        })
    }
}

impl Aura {
    /// Whether aura is applicable to the target
    pub fn is_applicable(&self, unit: &Unit, target: &Unit) -> bool {
        if let Some(radius) = self.radius {
            unit.position.distance(&target.position) <= radius
        } else {
            false
        }
    }
}

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

    status.vars.insert(VarName::StackCounter, 1);
    match &status.status.stacking {
        StatusStacking::Independent => {
            let id = status.id;
            all_statuses.push(status);
            return id;
        }
        StatusStacking::Refresh => {
            return replace(status, all_statuses, |s| {
                s.time = s.status.duration.map(Into::into);
                s.id
            })
        }
        StatusStacking::Count => {
            return replace(status, all_statuses, |s| {
                *s.vars.entry(VarName::StackCounter).or_insert(0) += 1;
                s.id
            })
        }
        StatusStacking::CountRefresh => {
            return replace(status, all_statuses, |s| {
                s.time = s.status.duration.map(Into::into);
                *s.vars.entry(VarName::StackCounter).or_insert(0) += 1;
                s.id
            })
        }
    }
}