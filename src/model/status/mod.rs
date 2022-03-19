use super::*;

#[derive(Debug, Serialize, Deserialize, Copy, Clone, Eq, PartialEq, Hash)]
pub enum StatusType {
    Freeze,
    Stun,
    Shield,
    Slow,
    Protection,
    Other,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum Status {
    Freeze,
    Stun {
        time: Time,
    },
    Shield,
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
            Self::Stun { .. } => StatusType::Stun,
            Self::Shield => StatusType::Shield,
            Self::Slow { .. } => StatusType::Slow,
            Self::Protection { .. } => StatusType::Protection,
            Self::Aura { .. } | Self::Modifier(..) | Status::DetectAttachedStatus { .. } => {
                StatusType::Other
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Aura {
    pub distance: Option<Coord>,
    pub alliance: Option<Alliance>, // TODO: Filter
    pub status: Box<Status>,
}
