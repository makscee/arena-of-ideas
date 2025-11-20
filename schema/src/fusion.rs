use super::*;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, AsRefStr)]
pub enum FusionType {
    StickFront,
    StickBack,
    PushBack,
}

impl FusionType {
    pub fn description(&self) -> &str {
        match self {
            FusionType::StickFront => "Take source trigger with combined actions",
            FusionType::StickBack => "Take target trigger with combined actions",
            FusionType::PushBack => "Keep both triggers and reactions",
        }
    }
}
