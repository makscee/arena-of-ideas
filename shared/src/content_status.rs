use serde::{Deserialize, Serialize};

/// Lifecycle status of a content entity (ability or unit).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContentStatus {
    /// Player is still editing with AI
    Draft,
    /// Submitted for community review
    Incubator,
    /// In the active game rotation
    Active,
    /// Rotated out of the active pool
    Retired,
}

impl std::fmt::Display for ContentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContentStatus::Draft => write!(f, "Draft"),
            ContentStatus::Incubator => write!(f, "Incubator"),
            ContentStatus::Active => write!(f, "Active"),
            ContentStatus::Retired => write!(f, "Retired"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_status_serde_roundtrip() {
        for status in [
            ContentStatus::Draft,
            ContentStatus::Incubator,
            ContentStatus::Active,
            ContentStatus::Retired,
        ] {
            let json = serde_json::to_string(&status).unwrap();
            let deserialized: ContentStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(status, deserialized);
        }
    }
}
