use super::*;

#[derive(Debug, Serialize, Deserialize, Copy, Clone, Eq, PartialEq, Hash)]
pub enum StatusType {
    Freeze,
    Stun,
    Shield,
    Invulnerability,
    Slow,
    Protection,
    DeathRattle,
    Other,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "status_type")]
pub enum Status {
    Freeze,
    Stun,
    Shield,
    Invulnerability,
    Slow {
        percent: f32,
    },
    Modifier(Modifier),
    Aura(Aura),
    Protection {
        percent: f32,
    },
    DetectAttachedStatus {
        on: TargetFilter,
        status: StatusType,
        effect: Effect,
    },
    Taunt {
        radius: Coord,
    },
    DeathRattle(Effect),
    BattleCry(Effect),
    Kill(UnitKillTrigger),
    Injured(UnitTakeDamageTrigger),
    ShieldBroken(UnitShieldBrokenTrigger),
    Gain(Effect),
    Scavenge {
        who: TargetFilter,
        effect: Effect,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AttachedStatus {
    #[serde(flatten)]
    pub status: Status,
    pub time: Option<Time>,
}

impl Status {
    pub fn r#type(&self) -> StatusType {
        match self {
            Self::Freeze => StatusType::Freeze,
            Self::Stun => StatusType::Stun,
            Self::Shield => StatusType::Shield,
            Self::Invulnerability => StatusType::Invulnerability,
            Self::Slow { .. } => StatusType::Slow,
            Self::Protection { .. } => StatusType::Protection,
            Self::DeathRattle { .. } => StatusType::DeathRattle,
            Self::BattleCry { .. }
            | Self::Aura { .. }
            | Self::Modifier { .. }
            | Self::DetectAttachedStatus { .. }
            | Self::Taunt { .. }
            | Self::Kill { .. }
            | Self::ShieldBroken { .. }
            | Self::Injured { .. }
            | Self::Gain { .. }
            | Self::Scavenge { .. } => StatusType::Other,
        }
    }
    pub fn walk_effects_mut(&mut self, f: &mut impl FnMut(&mut Effect)) {
        match self {
            Status::Freeze => {}
            Status::Stun => {}
            Status::Shield => {}
            Status::Invulnerability => {}
            Status::Slow { .. } => {}
            Status::Modifier { .. } => {}
            Status::Aura(Aura { status, .. }) => status.walk_effects_mut(f),
            Status::Protection { .. } => {}
            Status::DetectAttachedStatus { effect, .. } => effect.walk_mut(f),
            Status::Taunt { .. } => {}
            Status::DeathRattle(effect) => effect.walk_mut(f),
            Status::BattleCry(effect) => effect.walk_mut(f),
            Status::Kill(trigger) => trigger.effect.walk_mut(f),
            Status::Injured(trigger) => trigger.effect.walk_mut(f),
            Status::ShieldBroken(_) => {}
            Status::Gain(effect) => effect.walk_mut(f),
            Status::Scavenge { effect, .. } => effect.walk_mut(f),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Aura {
    pub distance: Option<Coord>,
    pub alliance: Option<Alliance>, // TODO: Filter
    pub status: Box<Status>,
}
