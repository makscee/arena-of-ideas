use super::*;

#[derive(Debug, Serialize, Deserialize, Copy, Clone, Eq, PartialEq, Hash)]
pub enum StatusType {
    Freeze,
    Stun,
    Shield,
    Slow,
    Modifier,
    Aura,
    Protection,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(tag = "type")]
pub enum Status {
    Freeze,
    Stun { time: Time },
    Shield,
    Slow { percent: f32 },
    Modifier(Modifier),
    Aura(Aura),
    Protection { percent: f32 },
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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
            Self::Aura { .. } => StatusType::Aura,
            Self::Modifier(..) => StatusType::Modifier,
            Self::Protection { .. } => StatusType::Protection,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Aura {
    pub distance: Option<Coord>,
    pub alliance: Option<Alliance>, // TODO: Filter
    pub status: Box<Status>,
}
