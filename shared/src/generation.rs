use serde::{Deserialize, Serialize};

/// What kind of content is being generated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GenTargetKind {
    Ability,
    Unit,
}

/// Status of a generation request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GenStatus {
    Pending,
    Processing,
    Done,
    Failed,
}

/// A request for AI to generate content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenRequest {
    pub id: u64,
    pub target_kind: GenTargetKind,
    pub prompt: String,
    /// For ability breeding: first parent ability ID
    pub parent_a: Option<u64>,
    /// For ability breeding: second parent ability ID
    pub parent_b: Option<u64>,
    /// For refinement: existing entity being iterated on
    pub context_id: Option<u64>,
    pub status: GenStatus,
}

/// The result of an AI generation request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenResult {
    pub id: u64,
    pub request_id: u64,
    /// JSON blob matching the target_kind struct (Ability or Unit)
    pub data: String,
    /// AI's reasoning for the player to review
    pub explanation: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gen_request_serde_roundtrip() {
        let req = GenRequest {
            id: 1,
            target_kind: GenTargetKind::Ability,
            prompt: "Combine fire and theft".to_string(),
            parent_a: Some(10),
            parent_b: Some(20),
            context_id: None,
            status: GenStatus::Pending,
        };

        let json = serde_json::to_string(&req).unwrap();
        let deserialized: GenRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.target_kind, GenTargetKind::Ability);
        assert_eq!(deserialized.parent_a, Some(10));
        assert_eq!(deserialized.status, GenStatus::Pending);
    }

    #[test]
    fn gen_result_serde_roundtrip() {
        let result = GenResult {
            id: 1,
            request_id: 1,
            data: r#"{"name":"Ember Heist"}"#.to_string(),
            explanation: "Combined fire damage with theft mechanics".to_string(),
        };

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: GenResult = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.request_id, 1);
    }
}
