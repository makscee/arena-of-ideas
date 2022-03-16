use super::*;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(tag = "type")]
pub enum Status {
    Freeze,
    Stun { time: Time },
    Shield,
    Slow { percent: f32, time: Time },
    Modifier(Modifier),
    Aura(Aura),
}

impl Status {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Freeze => "Freeze",
            Self::Stun { .. } => "Stun",
            Self::Shield => "Shield",
            Self::Slow { .. } => "Slow",
            Self::Aura { .. } => "Aura",
            Self::Modifier(..) => "Modifier",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Aura {
    pub distance: Option<Coord>,
    pub alliance: Option<Alliance>, // TODO: Filter
    pub status: Box<Status>,
    pub time: Option<Time>,
}
